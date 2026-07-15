/// Shared face alignment utilities used by both Buffalo_L and SFace+YuNet pipelines.
///
/// Both models share the same detection→alignment→recognition pattern:
///   1. A detector finds faces and returns 5-landmark detections
///   2. A similarity transform maps detected landmarks to ArcFace canonical positions
///   3. Backward-mapping affine warp produces a 112×112 aligned crop
///   4. The aligned crop is fed to a recognition model
use super::letterbox;
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

/// Crops, and the faces they came from, in the same order.
///
/// They travel together because a face that cannot be aligned is dropped from both: pairing
/// crop `i` back against the caller's detection list would give an embedding the bounding
/// box of the face before it.
pub(crate) struct AlignedFaces {
    pub crops: Array<f32, Ix4>,
    pub faces: Vec<FaceDetection>,
}

impl AlignedFaces {
    fn collect(pairs: Vec<(FaceDetection, Array<f32, Ix3>)>) -> Self {
        let mut crops = Array::zeros((pairs.len(), 3, 112, 112));
        let mut faces = Vec::with_capacity(pairs.len());

        for (i, (face, crop)) in pairs.into_iter().enumerate() {
            crops.slice_mut(ndarray::s![i, .., .., ..]).assign(&crop);
            faces.push(face);
        }

        Self { crops, faces }
    }

    pub fn len(&self) -> usize {
        self.faces.len()
    }
}

pub(crate) fn crop_and_align_faces(
    detections: &[FaceDetection],
    image: ndarray::ArrayView<f32, Ix4>,
) -> AlignedFaces {
    let pairs = detections
        .par_iter()
        .filter_map(|detection| match align_face(detection, image) {
            Ok(crop) => Some((detection.clone(), crop)),
            Err(e) => {
                tracing::warn!("skipping a face that could not be aligned: {e}");
                None
            }
        })
        .collect();

    AlignedFaces::collect(pairs)
}

/// Least squares over all 5 landmarks (Umeyama), the warp the recognition model was trained
/// on. An eyes-only fit takes its scale from the inter-ocular distance, which foreshortens
/// with yaw while eye-to-mouth does not, so a turned head comes out over-zoomed.
pub(crate) fn estimate_similarity_transform(
    src: &Array2<f32>,
    dst: &Array2<f32>,
) -> Result<Array2<f32>, AIProxyError> {
    let n = src.nrows();
    if n < 2 || dst.nrows() != n {
        return Err(AIProxyError::ModelProviderPreprocessingError(format!(
            "need at least 2 matching landmarks, got src {n} and dst {}",
            dst.nrows()
        )));
    }

    let centroid = |points: &Array2<f32>| {
        let n = points.nrows() as f32;
        (points.column(0).sum() / n, points.column(1).sum() / n)
    };
    let (src_cx, src_cy) = centroid(src);
    let (dst_cx, dst_cy) = centroid(dst);

    let mut variance = 0.0;
    let mut a = 0.0;
    let mut b = 0.0;
    for i in 0..n {
        let (sx, sy) = (src[[i, 0]] - src_cx, src[[i, 1]] - src_cy);
        let (dx, dy) = (dst[[i, 0]] - dst_cx, dst[[i, 1]] - dst_cy);
        variance += sx * sx + sy * sy;
        a += sx * dx + sy * dy;
        b += sx * dy - sy * dx;
    }

    if variance < 1e-6 {
        return Err(AIProxyError::ModelProviderPreprocessingError(
            "degenerate landmarks: every point is in the same place".to_string(),
        ));
    }

    let (a, b) = (a / variance, b / variance);
    let tx = dst_cx - (a * src_cx - b * src_cy);
    let ty = dst_cy - (b * src_cx + a * src_cy);

    Array2::from_shape_vec((2, 3), vec![a, -b, tx, b, a, ty])
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

/// Align a face out of the original image. `landmarks` must be in its coordinates.
///
/// Output is normalized the way the recognition model expects: `(x - 127.5) / 127.5`, RGB.
pub(crate) fn align_face_from_original(
    image: &image::RgbImage,
    landmarks: &[[f32; 2]; 5],
) -> Result<Array<f32, Ix3>, AIProxyError> {
    const SIZE: usize = 112;
    const MEAN: f32 = 127.5;
    const STD: f32 = 127.5;

    let src = Array2::from_shape_vec(
        (5, 2),
        landmarks.iter().flat_map(|&[x, y]| [x, y]).collect(),
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
    let (a, b, tx) = (transform[[0, 0]], transform[[0, 1]], transform[[0, 2]]);
    let (c, d, ty) = (transform[[1, 0]], transform[[1, 1]], transform[[1, 2]]);

    let det = a * d - b * c;
    if det.abs() < 1e-9 {
        return Err(AIProxyError::ModelProviderPreprocessingError(
            "singular alignment transform".to_string(),
        ));
    }

    let (inv_a, inv_b, inv_c, inv_d) = (d / det, -b / det, -c / det, a / det);
    let inv_tx = -(inv_a * tx + inv_b * ty);
    let inv_ty = -(inv_c * tx + inv_d * ty);

    let (width, height) = (image.width() as i64, image.height() as i64);
    let raw = image.as_raw();
    let mut out = Array::zeros((3, SIZE, SIZE));

    for y in 0..SIZE {
        let yf = y as f32;
        for x in 0..SIZE {
            let xf = x as f32;
            let sx = inv_a * xf + inv_b * yf + inv_tx;
            let sy = inv_c * xf + inv_d * yf + inv_ty;

            // Outside the image reads as black, as `warpAffine(borderValue=0)` does.
            let (x0, y0) = (sx.floor() as i64, sy.floor() as i64);
            let (fx, fy) = (sx - x0 as f32, sy - y0 as f32);

            let mut channels = [0.0f32; 3];
            for (dy, wy) in [(0, 1.0 - fy), (1, fy)] {
                for (dx, wx) in [(0, 1.0 - fx), (1, fx)] {
                    let (px, py) = (x0 + dx, y0 + dy);
                    if px >= 0 && py >= 0 && px < width && py < height {
                        let weight = wx * wy;
                        let base = ((py as usize * width as usize) + px as usize) * 3;
                        for (channel, value) in channels.iter_mut().enumerate() {
                            *value += raw[base + channel] as f32 * weight;
                        }
                    }
                }
            }

            for (channel, value) in channels.iter().enumerate() {
                out[[channel, y, x]] = (value - MEAN) / STD;
            }
        }
    }

    Ok(out)
}

/// Align every detection out of the original image. Detections are in letterbox coordinates.
pub(crate) fn crop_and_align_from_original(
    detections: &[FaceDetection],
    image: &image::RgbImage,
    letterbox_size: u32,
) -> AlignedFaces {
    let (width, height) = image.dimensions();
    let geometry = letterbox::params(width, height, letterbox_size);

    // SCRFD can emit collapsed landmarks for a tiny or blurred face. Drop that face; do not
    // fail the request.
    let pairs = detections
        .par_iter()
        .filter_map(|detection| {
            let mut landmarks = [[0.0f32; 2]; 5];
            for (out, &point) in landmarks.iter_mut().zip(detection.landmarks.iter()) {
                *out = geometry.to_original(point);
            }
            match align_face_from_original(image, &landmarks) {
                Ok(crop) => Some((detection.clone(), crop)),
                Err(e) => {
                    tracing::warn!("skipping a face that could not be aligned: {e}");
                    None
                }
            }
        })
        .collect();

    AlignedFaces::collect(pairs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_fit_recovers_a_known_transform() {
        let angle: f32 = 0.3;
        let (scale, tx, ty) = (2.5f32, 17.0f32, -9.0f32);

        let src = Array2::from_shape_vec(
            (5, 2),
            ARCFACE_REFERENCE
                .iter()
                .flat_map(|&[x, y]| [x, y])
                .collect(),
        )
        .unwrap();

        let mut dst = Array2::zeros((5, 2));
        for i in 0..5 {
            let (x, y) = (src[[i, 0]], src[[i, 1]]);
            dst[[i, 0]] = scale * (x * angle.cos() - y * angle.sin()) + tx;
            dst[[i, 1]] = scale * (x * angle.sin() + y * angle.cos()) + ty;
        }

        let m = estimate_similarity_transform(&src, &dst).unwrap();

        assert!(
            (m[[0, 0]] - scale * angle.cos()).abs() < 1e-3,
            "a: {}",
            m[[0, 0]]
        );
        assert!(
            (m[[1, 0]] - scale * angle.sin()).abs() < 1e-3,
            "b: {}",
            m[[1, 0]]
        );
        assert!((m[[0, 2]] - tx).abs() < 1e-2, "tx: {}", m[[0, 2]]);
        assert!((m[[1, 2]] - ty).abs() < 1e-2, "ty: {}", m[[1, 2]]);
    }

    /// The bug this guards: when a face in the middle is dropped, the survivors must keep
    /// their own bounding boxes, not inherit the previous face's.
    #[test]
    fn a_dropped_face_does_not_shift_the_rest() {
        let image = image::RgbImage::from_pixel(200, 200, image::Rgb([128, 128, 128]));

        let face = |lm: [[f32; 2]; 5]| FaceDetection {
            bbox: [lm[0][0], lm[0][1], lm[1][0], lm[3][1]],
            landmarks: lm,
            confidence: 0.9,
        };
        // Two real faces around a degenerate one (every landmark on the same point).
        let good_a = face([
            [40.0, 40.0],
            [70.0, 40.0],
            [55.0, 55.0],
            [42.0, 75.0],
            [68.0, 75.0],
        ]);
        let bad = face([[10.0, 10.0]; 5]);
        let good_b = face([
            [120.0, 120.0],
            [150.0, 120.0],
            [135.0, 135.0],
            [122.0, 155.0],
            [148.0, 155.0],
        ]);

        let out = crop_and_align_from_original(&[good_a.clone(), bad, good_b.clone()], &image, 200);

        assert_eq!(out.len(), 2, "the degenerate face must be dropped");
        assert_eq!(
            out.faces[0].bbox, good_a.bbox,
            "survivor 0 kept its own box"
        );
        assert_eq!(
            out.faces[1].bbox, good_b.bbox,
            "survivor 1 did not inherit the dropped box"
        );
    }

    #[test]
    fn the_fit_uses_every_landmark() {
        let src = Array2::from_shape_vec(
            (5, 2),
            ARCFACE_REFERENCE
                .iter()
                .flat_map(|&[x, y]| [x, y])
                .collect(),
        )
        .unwrap();

        let mut moved = src.clone();
        moved[[3, 1]] += 12.0; // left mouth corner only
        moved[[4, 1]] += 12.0; // right mouth corner only

        let baseline = estimate_similarity_transform(&src, &src).unwrap();
        let shifted = estimate_similarity_transform(&moved, &src).unwrap();

        assert!(
            (baseline[[1, 2]] - shifted[[1, 2]]).abs() > 1.0,
            "an eye-only fit would ignore the mouth; ty moved by {}",
            (baseline[[1, 2]] - shifted[[1, 2]]).abs()
        );
    }
}
