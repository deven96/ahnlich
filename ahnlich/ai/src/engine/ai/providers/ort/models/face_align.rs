/// Shared face alignment utilities used by both Buffalo_L and SFace+YuNet pipelines.
///
/// Both models share the same detection→alignment→recognition pattern:
///   1. A detector finds faces and returns 5-landmark detections
///   2. A similarity transform maps detected landmarks to ArcFace canonical positions
///   3. Backward-mapping affine warp produces a 112×112 aligned crop
///   4. The aligned crop is fed to a recognition model
use crate::error::AIProxyError;
use ndarray::{Array, Array2, Ix3, Ix4};
use rayon::prelude::*;
use std::mem::size_of;

/// A detected face: bounding box, 5 facial landmarks, and confidence score.
///
/// Landmarks order: left_eye, right_eye, nose_tip, left_mouth, right_mouth.
/// All coordinates are in pixels relative to the input image.
#[derive(Debug, Clone)]
pub(crate) struct FaceDetection {
    pub bbox: [f32; 4],           // [x1, y1, x2, y2]
    pub landmarks: [[f32; 2]; 5], // [[x, y]; 5]
    pub confidence: f32,
}

/// ArcFace standard reference landmark positions in a 112×112 aligned output.
///
/// These are the canonical positions that both Buffalo_L (ResNet50) and SFace
/// expect after alignment. Using the same reference for both models ensures
/// embeddings are geometrically comparable.
pub(crate) const ARCFACE_REFERENCE: [[f32; 2]; 5] = [
    [38.2946, 51.6963], // left eye   (30.2946 + 8.0)
    [73.5318, 51.5014], // right eye  (65.5318 + 8.0)
    [56.0252, 71.7366], // nose tip   (48.0252 + 8.0)
    [41.5493, 92.3655], // left mouth (33.5493 + 8.0)
    [70.7299, 92.2041], // right mouth(62.7299 + 8.0)
];

/// Non-Maximum Suppression: collapse overlapping detections to the highest-confidence one.
///
/// Sorts by confidence descending, then greedily keeps detections whose IoU with any
/// already-kept detection is below `iou_threshold`.
pub(crate) fn apply_nms(
    mut detections: Vec<FaceDetection>,
    iou_threshold: f32,
) -> Vec<FaceDetection> {
    if detections.len() <= 1 {
        return detections;
    }

    detections.sort_unstable_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

    let n = detections.len();
    let mut suppressed = vec![false; n];
    let mut keep = Vec::with_capacity(n);

    for i in 0..n {
        if suppressed[i] {
            continue;
        }
        keep.push(detections[i].clone());
        for j in (i + 1)..n {
            if !suppressed[j]
                && calculate_iou(&detections[i].bbox, &detections[j].bbox) > iou_threshold
            {
                suppressed[j] = true;
            }
        }
    }
    keep
}

/// Intersection-over-Union for two [x1, y1, x2, y2] boxes.
#[inline]
pub(crate) fn calculate_iou(box1: &[f32; 4], box2: &[f32; 4]) -> f32 {
    let inter_x1 = box1[0].max(box2[0]);
    let inter_y1 = box1[1].max(box2[1]);
    let inter_x2 = box1[2].min(box2[2]);
    let inter_y2 = box1[3].min(box2[3]);

    let inter_area = (inter_x2 - inter_x1).max(0.0) * (inter_y2 - inter_y1).max(0.0);
    let b1_area = (box1[2] - box1[0]) * (box1[3] - box1[1]);
    let b2_area = (box2[2] - box2[0]) * (box2[3] - box2[1]);
    let union_area = b1_area + b2_area - inter_area;

    if union_area > 0.0 {
        inter_area / union_area
    } else {
        0.0
    }
}

/// Align a detected face to the ArcFace 112×112 canonical pose.
///
/// Estimates a similarity transform (rotation + uniform scale + translation)
/// from the 5 detected landmarks to `ARCFACE_REFERENCE`, then warps the image.
pub(crate) fn align_face(
    detection: &FaceDetection,
    image: ndarray::ArrayView<f32, Ix4>,
) -> Result<Array<f32, Ix3>, AIProxyError> {
    let src = Array2::from_shape_vec(
        (5, 2),
        detection
            .landmarks
            .iter()
            .flat_map(|&[x, y]| [x, y])
            .collect(),
    )
    .map_err(|e| AIProxyError::ModelProviderPreprocessingError(e.to_string()))?;

    let dst = Array2::from_shape_vec(
        (5, 2),
        ARCFACE_REFERENCE
            .iter()
            .flat_map(|&[x, y]| [x, y])
            .collect(),
    )
    .map_err(|e| AIProxyError::ModelProviderPreprocessingError(e.to_string()))?;

    let transform = estimate_similarity_transform(&src, &dst)?;
    warp_affine(image, &transform, 112, 112)
}

/// Crop and align all detections from one image into a (N, 3, 112, 112) batch.
///
/// Processes faces in parallel using Rayon when there are multiple faces detected.
/// For single-face images, uses sequential processing to avoid parallelization overhead.
pub(crate) fn crop_and_align_faces(
    detections: &[FaceDetection],
    image: ndarray::ArrayView<f32, Ix4>,
) -> Result<Array<f32, Ix4>, AIProxyError> {
    if detections.is_empty() {
        return Ok(Array::zeros((0, 3, 112, 112)));
    }

    let num_faces = detections.len();

    // For single face, use sequential processing (avoid parallelization overhead)
    if num_faces == 1 {
        let aligned = align_face(&detections[0], image)?;
        let mut batch = Array::zeros((1, 3, 112, 112));
        batch.slice_mut(ndarray::s![0, .., .., ..]).assign(&aligned);
        return Ok(batch);
    }

    // For multiple faces, process in parallel
    let aligned_faces: Result<Vec<_>, AIProxyError> = detections
        .par_iter()
        .map(|det| align_face(det, image))
        .collect();

    let aligned_faces = aligned_faces?;

    let mut batch = Array::zeros((num_faces, 3, 112, 112));
    for (i, aligned) in aligned_faces.into_iter().enumerate() {
        batch.slice_mut(ndarray::s![i, .., .., ..]).assign(&aligned);
    }
    Ok(batch)
}

/// Estimate a 2×3 similarity transform matrix mapping `src` landmarks to `dst`.
///
/// Uses eye-vector angle and scale, then solves for translation from the eye midpoints.
/// This is a simplified Umeyama — correct for the near-frontal face case.
///
/// The scale term computed from the full point-cloud norms is intentionally dropped:
/// scale is already captured by the eye-vector ratio, and the full-cloud scale
/// adds noise when mouth/nose points are misdetected.
pub(crate) fn estimate_similarity_transform(
    src: &Array2<f32>,
    dst: &Array2<f32>,
) -> Result<Array2<f32>, AIProxyError> {
    // Eye-to-eye vectors
    let src_dx = src[[1, 0]] - src[[0, 0]];
    let src_dy = src[[1, 1]] - src[[0, 1]];
    let dst_dx = dst[[1, 0]] - dst[[0, 0]];
    let dst_dy = dst[[1, 1]] - dst[[0, 1]];

    // Scale: ratio of eye distances
    let src_eye_dist = (src_dx * src_dx + src_dy * src_dy).sqrt();
    let dst_eye_dist = (dst_dx * dst_dx + dst_dy * dst_dy).sqrt();
    let scale = if src_eye_dist > 1e-6 {
        dst_eye_dist / src_eye_dist
    } else {
        1.0
    };

    // Rotation angle between eye vectors
    let angle = dst_dy.atan2(dst_dx) - src_dy.atan2(src_dx);
    let cos_a = angle.cos() * scale;
    let sin_a = angle.sin() * scale;

    // Translation: map source eye midpoint to destination eye midpoint
    let src_eye_cx = (src[[0, 0]] + src[[1, 0]]) / 2.0;
    let src_eye_cy = (src[[0, 1]] + src[[1, 1]]) / 2.0;
    let dst_eye_cx = (dst[[0, 0]] + dst[[1, 0]]) / 2.0;
    let dst_eye_cy = (dst[[0, 1]] + dst[[1, 1]]) / 2.0;

    let tx = dst_eye_cx - (src_eye_cx * cos_a - src_eye_cy * sin_a);
    let ty = dst_eye_cy - (src_eye_cx * sin_a + src_eye_cy * cos_a);

    Array2::from_shape_vec((2, 3), vec![cos_a, -sin_a, tx, sin_a, cos_a, ty])
        .map_err(|e| AIProxyError::ModelProviderPreprocessingError(e.to_string()))
}

/// Warp an image with a 2×3 affine transform using backward-mapping + bilinear interpolation.
///
/// For each destination pixel (dx, dy) we compute the inverse transform to find the
/// source coordinate, then bilinearly interpolate from the 4 surrounding source pixels.
///
/// **Performance note**: `inv_b * dst_yf + inv_tx` and `inv_d * dst_yf + inv_ty` are
/// row-level constants hoisted outside the inner x-loop, saving two multiplications
/// per output pixel (112 × 112 × 3 ≈ 37k pixels per face).
pub(crate) fn warp_affine(
    image: ndarray::ArrayView<f32, Ix4>,
    transform: &Array2<f32>,
    output_width: usize,
    output_height: usize,
) -> Result<Array<f32, Ix3>, AIProxyError> {
    let img_shape = image.shape();
    let channels = img_shape[1];
    let src_height = img_shape[2];
    let src_width = img_shape[3];

    let output_pixels = channels * output_height * output_width;
    let estimated_bytes = output_pixels * size_of::<f32>() + 64;
    utils::allocator::check_memory_available(estimated_bytes)
        .map_err(|e| AIProxyError::Allocation(e.into()))?;

    let a = transform[[0, 0]];
    let b = transform[[0, 1]];
    let tx = transform[[0, 2]];
    let c = transform[[1, 0]];
    let d = transform[[1, 1]];
    let ty = transform[[1, 2]];

    let det = a * d - b * c;
    if det.abs() < 1e-6 {
        return Err(AIProxyError::ModelProviderPreprocessingError(
            "Singular transformation matrix in warp_affine".to_string(),
        ));
    }

    // Inverse 2×2 + translation
    let inv_a = d / det;
    let inv_b = -b / det;
    let inv_c = -c / det;
    let inv_d = a / det;
    let inv_tx = (b * ty - d * tx) / det;
    let inv_ty = (c * tx - a * ty) / det;

    let src_w_limit = (src_width - 1) as f32;
    let src_h_limit = (src_height - 1) as f32;

    let mut output = Array::zeros((channels, output_height, output_width));

    // Process rows in parallel for better performance on larger transforms
    // Each row is independent and can be computed simultaneously
    let row_data: Vec<Vec<Vec<f32>>> = (0..output_height)
        .into_par_iter()
        .map(|dst_y| {
            let dst_yf = dst_y as f32;
            // Hoist row-level constants out of inner x-loop
            let row_src_x_base = inv_b * dst_yf + inv_tx;
            let row_src_y_base = inv_d * dst_yf + inv_ty;

            let mut row_pixels = vec![vec![0.0f32; output_width]; channels];

            // Clippy suggests enumerate() here, but we need dst_x for coordinate transformation
            // calculations (dst_xf = dst_x as f32), not just array indexing
            #[allow(clippy::needless_range_loop)]
            for dst_x in 0..output_width {
                let dst_xf = dst_x as f32;
                let src_x = inv_a * dst_xf + row_src_x_base;
                let src_y = inv_c * dst_xf + row_src_y_base;

                if src_x >= 0.0 && src_x < src_w_limit && src_y >= 0.0 && src_y < src_h_limit {
                    let x0 = src_x as usize;
                    let y0 = src_y as usize;
                    let x1 = x0 + 1;
                    let y1 = y0 + 1;
                    let dx = src_x - x0 as f32;
                    let dy = src_y - y0 as f32;
                    let dx1 = 1.0 - dx;
                    let dy1 = 1.0 - dy;

                    for ch in 0..channels {
                        let p = dx1 * (dy1 * image[[0, ch, y0, x0]] + dy * image[[0, ch, y1, x0]])
                            + dx * (dy1 * image[[0, ch, y0, x1]] + dy * image[[0, ch, y1, x1]]);
                        row_pixels[ch][dst_x] = p;
                    }
                }
            }
            row_pixels
        })
        .collect();

    // Copy parallel results back to output array
    for (dst_y, row_pixels) in row_data.into_iter().enumerate() {
        for ch in 0..channels {
            for dst_x in 0..output_width {
                output[[ch, dst_y, dst_x]] = row_pixels[ch][dst_x];
            }
        }
    }

    Ok(output)
}
