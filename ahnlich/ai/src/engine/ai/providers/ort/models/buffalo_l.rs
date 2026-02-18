use super::super::InnerAIExecutionProvider;
use super::super::executor::ExecutorWithSessionCache;
use super::super::inference_model::ORTInferenceModel;
use crate::engine::ai::models::{ModelInput, ModelResponse};
use crate::error::AIProxyError;
use ahnlich_types::ai::execution_provider::ExecutionProvider as AIExecutionProvider;
use ahnlich_types::keyval::StoreKey;
use hf_hub::api::sync::Api;
use ndarray::{Array, Array2, Axis, Ix2, Ix3, Ix4, s};
use ort::Session;
use std::future::Future;
use std::mem::size_of;
use std::pin::Pin;

/// Buffalo_L: Multi-stage face detection + recognition model from InsightFace
///
/// This model performs face detection using RetinaFace and face recognition using ResNet50.
/// Pipeline: Image → RetinaFace detection → Face alignment → ResNet50 recognition → Embeddings
///
/// Key features:
/// - RetinaFace: Multi-scale face detection with 3 FPN (Feature Pyramid Network) levels
/// - Face alignment: Uses 5 facial landmarks to normalize face pose
/// - ResNet50: Generates 512-dimensional face embeddings
/// - OneToMany: Returns multiple face embeddings per input image
pub(crate) struct BuffaloLModel {
    detection_cache: ExecutorWithSessionCache, // RetinaFace model session
    recognition_cache: ExecutorWithSessionCache, // ResNet50 model session
    model_batch_size: usize,
    anchors: Vec<Anchor>, // Pre-generated anchor boxes for RetinaFace bbox decoding
}

/// Anchor box for RetinaFace object detection
///
/// RetinaFace uses anchor-based detection where predictions are offsets relative to
/// pre-defined anchor boxes at different scales and positions in the image.
#[derive(Debug, Clone)]
struct Anchor {
    x: f32,      // Center x coordinate in pixels
    y: f32,      // Center y coordinate in pixels
    width: f32,  // Anchor width in pixels
    height: f32, // Anchor height in pixels
}

impl BuffaloLModel {
    #[tracing::instrument(skip_all)]
    pub async fn build(
        api: Api,
        session_profiling: bool,
    ) -> Result<Box<dyn ORTInferenceModel>, AIProxyError> {
        let repo = api.model("immich-app/buffalo_l".to_string());

        let det_file = repo
            .get("detection/model.onnx")
            .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?;
        let det_cache = ExecutorWithSessionCache::new(det_file, session_profiling);
        det_cache
            .try_get_with(InnerAIExecutionProvider::CPU)
            .await?;

        let rec_file = repo
            .get("recognition/model.onnx")
            .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?;
        let rec_cache = ExecutorWithSessionCache::new(rec_file, session_profiling);
        rec_cache
            .try_get_with(InnerAIExecutionProvider::CPU)
            .await?;

        // Generate anchors for RetinaFace detection model
        // Input size is 640x640, with 3 FPN scales
        let anchors = Self::generate_anchors(640, 640);

        Ok(Box::new(Self {
            detection_cache: det_cache,
            recognition_cache: rec_cache,
            model_batch_size: 16,
            anchors,
        }))
    }
}

impl ORTInferenceModel for BuffaloLModel {
    fn infer_batch(
        &self,
        input: ModelInput,
        execution_provider: Option<AIExecutionProvider>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ModelResponse>, AIProxyError>> + Send + '_>> {
        Box::pin(async move {
            match input {
                ModelInput::Images(images) => {
                    self.detect_and_recognize_batch(images, execution_provider)
                        .await
                }
                ModelInput::Texts(_) => Err(AIProxyError::AIModelNotSupported {
                    model_name: "Buffalo_L (image-only model)".to_string(),
                }),
            }
        })
    }

    fn batch_size(&self) -> usize {
        self.model_batch_size
    }
}

impl BuffaloLModel {
    /// Generate anchor boxes for RetinaFace at multiple FPN scales
    ///
    /// RetinaFace uses a Feature Pyramid Network (FPN) with 3 scales to detect faces
    /// at different sizes. Each scale has its own stride (downsampling factor) and
    /// base anchor sizes. This creates a dense grid of anchors across the image.
    ///
    /// For a 640x640 input:
    /// - Scale 0 (stride=8):  80x80 grid = 12,800 anchors (2 sizes each)
    /// - Scale 1 (stride=16): 40x40 grid = 3,200 anchors (2 sizes each)
    /// - Scale 2 (stride=32): 20x20 grid = 800 anchors (2 sizes each)
    /// Total: 16,800 anchors
    #[tracing::instrument]
    fn generate_anchors(input_height: usize, input_width: usize) -> Vec<Anchor> {
        // FPN configuration: (stride, base_sizes)
        // Smaller strides detect smaller faces, larger strides detect larger faces
        let fpn_configs = vec![
            (8, vec![16.0, 32.0]),    // Scale 0: Small faces
            (16, vec![64.0, 128.0]),  // Scale 1: Medium faces
            (32, vec![256.0, 512.0]), // Scale 2: Large faces
        ];

        let mut all_anchors = Vec::new();

        for (stride, base_sizes) in fpn_configs {
            // Calculate feature map dimensions after downsampling
            let feat_h = input_height / stride;
            let feat_w = input_width / stride;

            // Generate anchors at each feature map position
            for i in 0..feat_h {
                for j in 0..feat_w {
                    // Center anchors at the middle of each stride cell
                    let center_x = (j as f32 + 0.5) * stride as f32;
                    let center_y = (i as f32 + 0.5) * stride as f32;

                    // Create multiple anchor boxes at this position with different sizes
                    for &base_size in &base_sizes {
                        all_anchors.push(Anchor {
                            x: center_x,
                            y: center_y,
                            width: base_size,
                            height: base_size,
                        });
                    }
                }
            }
        }

        all_anchors
    }

    /// Main pipeline: Detect faces in all images, then batch-recognize all faces together
    ///
    /// This is a two-stage process:
    /// 1. Detection stage: Run RetinaFace on each image to find face locations
    /// 2. Recognition stage: Batch all detected faces and run ResNet50 once
    ///
    /// Batching the recognition stage is crucial for performance - instead of running
    /// ResNet50 N times for N faces, we run it once with a batch of N faces.
    #[tracing::instrument(skip(self, images))]
    async fn detect_and_recognize_batch(
        &self,
        images: Array<f32, Ix4>,
        execution_provider: Option<AIExecutionProvider>,
    ) -> Result<Vec<ModelResponse>, AIProxyError> {
        let exec_prov = execution_provider
            .map(|ep| ep.into())
            .unwrap_or(InnerAIExecutionProvider::CPU);
        let det_session = self.detection_cache.try_get_with(exec_prov).await?;
        let rec_session = self.recognition_cache.try_get_with(exec_prov).await?;

        let batch_size = images.shape()[0];
        let mut all_cropped_faces: Vec<Array<f32, Ix3>> = Vec::new();
        let mut face_counts: Vec<usize> = Vec::with_capacity(batch_size);

        // Stage 1: Detect and extract faces from each input image
        for image_idx in 0..batch_size {
            let single_image = images.slice(s![image_idx..image_idx + 1, .., .., ..]);
            let detections = self.detect_faces(single_image.to_owned(), &det_session)?;

            if detections.is_empty() {
                face_counts.push(0);
                continue;
            }

            // Crop and align detected faces to canonical pose
            let cropped_faces = self.crop_faces(&detections, single_image)?;
            let num_faces = cropped_faces.shape()[0];
            face_counts.push(num_faces);

            // Collect all faces for batch recognition
            for face_idx in 0..num_faces {
                all_cropped_faces.push(cropped_faces.slice(s![face_idx, .., .., ..]).to_owned());
            }
        }

        // Stage 2: Batch recognize all detected faces in one forward pass
        let embeddings = if all_cropped_faces.is_empty() {
            Array::zeros((0, 512))
        } else {
            // Memory check: Stacking all detected faces into a single batch array
            // 112×112×3: Face alignment normalizes all faces to this fixed size (ArcFace standard)
            // + 64: ndarray struct overhead
            let num_faces = all_cropped_faces.len();
            let face_pixels = 112 * 112 * 3;
            let estimated_bytes = num_faces * face_pixels * size_of::<f32>() + 64;
            utils::allocator::check_memory_available(estimated_bytes)
                .map_err(|e| AIProxyError::Allocation(e.into()))?;

            let faces_batch = ndarray::stack(
                Axis(0),
                &all_cropped_faces
                    .iter()
                    .map(|f| f.view())
                    .collect::<Vec<_>>(),
            )
            .map_err(|e| AIProxyError::ModelProviderPreprocessingError(e.to_string()))?;

            self.recognize_faces(faces_batch, &rec_session)?
        };

        // Step 3: Map embeddings back to source images
        // Memory check: Building ModelResponse for each face embedding
        // 512: ResNet50 embedding dimension (fixed by model architecture)
        // + 64: Vec overhead for StoreKey
        let total_faces: usize = face_counts.iter().sum();
        let bytes_per_face =
            size_of::<ModelResponse>() + size_of::<StoreKey>() + (512 * size_of::<f32>()) + 64;
        let estimated_bytes = total_faces * bytes_per_face;
        utils::allocator::check_memory_available(estimated_bytes)
            .map_err(|e| AIProxyError::Allocation(e.into()))?;

        let mut results: Vec<ModelResponse> = Vec::with_capacity(batch_size);
        let mut embedding_offset = 0;

        for &num_faces in &face_counts {
            if num_faces == 0 {
                results.push(ModelResponse::OneToMany(vec![]));
            } else {
                let face_keys: Vec<StoreKey> = embeddings
                    .slice(s![embedding_offset..embedding_offset + num_faces, ..])
                    .axis_iter(Axis(0))
                    .map(|embedding| StoreKey {
                        key: embedding.to_vec(),
                    })
                    .collect();

                results.push(ModelResponse::OneToMany(face_keys));
                embedding_offset += num_faces;
            }
        }

        Ok(results)
    }
    #[tracing::instrument(skip(self, image, session))]
    fn detect_faces(
        &self,
        image: Array<f32, Ix4>,
        session: &Session,
    ) -> Result<Vec<FaceDetection>, AIProxyError> {
        // RetinaFace detection model expects "input.1" tensor (not "input")
        let session_inputs = ort::inputs!["input.1" => image.view()]
            .map_err(|e| AIProxyError::ModelProviderPreprocessingError(e.to_string()))?;

        let outputs = session
            .run(session_inputs)
            .map_err(|e| AIProxyError::ModelProviderRunInferenceError(e.to_string()))?;

        self.parse_detections(outputs)
    }

    /// Parse RetinaFace outputs and decode bounding boxes using anchor-based regression
    ///
    /// RetinaFace outputs 9 tensors (3 FPN scales × 3 outputs per scale):
    /// - Scores: Confidence that an anchor contains a face
    /// - Bbox deltas: Offsets to adjust anchor box to actual face location
    /// - Landmark deltas: Offsets for 5 facial landmarks (eyes, nose, mouth corners)
    ///
    /// The model outputs DELTAS (offsets), not absolute positions. We must decode
    /// them using the pre-generated anchors to get actual pixel coordinates.
    #[tracing::instrument(skip(self, outputs))]
    fn parse_detections(
        &self,
        outputs: ort::SessionOutputs,
    ) -> Result<Vec<FaceDetection>, AIProxyError> {
        let output_tensors: Vec<_> = outputs
            .values()
            .map(|value| value.try_extract_tensor::<f32>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;

        // TODO: (deven96) We need to think about how to pass custom special arguments to various
        // models ... perhaps given an always extensible KV which each model then validates against
        // or else picks a reasonable default.
        const CONFIDENCE_THRESHOLD: f32 = 0.5;

        // Memory check: Allocating Vec for face detection results
        // 50: Conservative upper bound (typical images have 1-10 faces, but group photos can have more)
        // + 64: Vec overhead
        let max_expected_faces = 50;
        let bytes_per_detection = size_of::<FaceDetection>();
        let estimated_bytes = max_expected_faces * bytes_per_detection + 64;
        utils::allocator::check_memory_available(estimated_bytes)
            .map_err(|e| AIProxyError::Allocation(e.into()))?;

        let mut all_detections = Vec::new();
        let mut anchor_offset = 0; // Track position in anchor array across scales

        // Process each FPN scale separately
        for scale_idx in 0..3 {
            // RetinaFace outputs are ordered: [score0, bbox0, landmark0, score1, bbox1, landmark1, ...]
            let score_idx = scale_idx * 3;
            let bbox_idx = score_idx + 1;
            let landmark_idx = score_idx + 2;

            let scores = &output_tensors[score_idx];
            let bbox_deltas = &output_tensors[bbox_idx];
            let landmark_deltas = &output_tensors[landmark_idx];

            let num_anchors = scores.shape()[0];

            let scores_slice = scores.as_slice().unwrap();
            let bbox_deltas_slice = bbox_deltas.as_slice().unwrap();
            let landmark_deltas_slice = landmark_deltas.as_slice().unwrap();

            // Check each anchor at this scale
            for i in 0..num_anchors {
                let confidence = scores_slice[i];

                if confidence < CONFIDENCE_THRESHOLD {
                    continue;
                }

                let anchor_idx = anchor_offset + i;
                let anchor = &self.anchors[anchor_idx];

                // Decode bbox from anchor + deltas using variance [0.1, 0.2]
                // This is the standard RetinaFace decoding formula
                // RetinaFace uses variance [0.1, 0.2] for center/size encoding
                let dx = bbox_deltas_slice[i * 4] * 0.1;
                let dy = bbox_deltas_slice[i * 4 + 1] * 0.1;
                let dw = bbox_deltas_slice[i * 4 + 2] * 0.2;
                let dh = bbox_deltas_slice[i * 4 + 3] * 0.2;

                let pred_cx = anchor.x + dx * anchor.width;
                let pred_cy = anchor.y + dy * anchor.height;
                let pred_w = anchor.width * dw.exp();
                let pred_h = anchor.height * dh.exp();

                // Convert to [x1, y1, x2, y2] format
                let bbox = [
                    pred_cx - pred_w / 2.0,
                    pred_cy - pred_h / 2.0,
                    pred_cx + pred_w / 2.0,
                    pred_cy + pred_h / 2.0,
                ];

                // Decode landmarks from anchor + deltas
                let mut landmarks = [[0.0f32; 2]; 5];
                for j in 0..5 {
                    let ldx = landmark_deltas_slice[i * 10 + j * 2] * 0.1;
                    let ldy = landmark_deltas_slice[i * 10 + j * 2 + 1] * 0.1;
                    landmarks[j][0] = anchor.x + ldx * anchor.width;
                    landmarks[j][1] = anchor.y + ldy * anchor.height;
                }

                all_detections.push(FaceDetection {
                    bbox,
                    landmarks,
                    confidence,
                });
            }

            anchor_offset += num_anchors;
        }

        // Apply Non-Maximum Suppression to remove duplicate detections
        // Multi-scale detection produces ~4-6 duplicates per face, NMS keeps the best one
        let nms_detections = self.apply_nms(all_detections, 0.4);
        Ok(nms_detections)
    }

    /// Non-Maximum Suppression (NMS): Remove duplicate face detections
    ///
    /// Multi-scale RetinaFace detection produces multiple overlapping boxes for the
    /// same face. NMS keeps only the highest-confidence detection and suppresses
    /// others that overlap significantly (IoU > threshold).
    ///
    /// Algorithm:
    /// 1. Sort detections by confidence (descending)
    /// 2. Keep the highest-confidence detection
    /// 3. Suppress all detections that overlap with it (IoU > threshold)
    /// 4. Repeat for remaining unsuppressed detections
    #[tracing::instrument(skip(self, detections))]
    fn apply_nms(
        &self,
        mut detections: Vec<FaceDetection>,
        iou_threshold: f32,
    ) -> Vec<FaceDetection> {
        if detections.is_empty() {
            return detections;
        }

        // Sort by confidence (highest first) - better detections take priority
        detections.sort_by(|a, b| b.confidence.partial_cmp(&a.confidence).unwrap());

        let mut keep = Vec::new();
        let mut suppressed = vec![false; detections.len()];

        for i in 0..detections.len() {
            if suppressed[i] {
                continue;
            }
            keep.push(detections[i].clone());

            // Suppress all lower-confidence detections that overlap significantly
            for j in (i + 1)..detections.len() {
                if suppressed[j] {
                    continue;
                }

                let iou = self.calculate_iou(&detections[i].bbox, &detections[j].bbox);
                if iou > iou_threshold {
                    suppressed[j] = true;
                }
            }
        }

        keep
    }

    #[tracing::instrument(skip(self))]
    fn calculate_iou(&self, box1: &[f32; 4], box2: &[f32; 4]) -> f32 {
        // Boxes are already in [x1, y1, x2, y2] format
        let b1_x1 = box1[0];
        let b1_y1 = box1[1];
        let b1_x2 = box1[2];
        let b1_y2 = box1[3];

        let b2_x1 = box2[0];
        let b2_y1 = box2[1];
        let b2_x2 = box2[2];
        let b2_y2 = box2[3];

        let inter_x1 = b1_x1.max(b2_x1);
        let inter_y1 = b1_y1.max(b2_y1);
        let inter_x2 = b1_x2.min(b2_x2);
        let inter_y2 = b1_y2.min(b2_y2);

        let inter_area = (inter_x2 - inter_x1).max(0.0) * (inter_y2 - inter_y1).max(0.0);

        let b1_area = (b1_x2 - b1_x1) * (b1_y2 - b1_y1);
        let b2_area = (b2_x2 - b2_x1) * (b2_y2 - b2_y1);

        let union_area = b1_area + b2_area - inter_area;

        if union_area > 0.0 {
            inter_area / union_area
        } else {
            0.0
        }
    }

    /// Extract and align all detected faces from an image
    ///
    /// Instead of simply cropping bounding boxes, this method performs proper face
    /// alignment using the 5 facial landmarks. Alignment normalizes the face pose,
    /// rotation, and scale, which significantly improves recognition accuracy.
    #[tracing::instrument(skip(self, detections, image))]
    fn crop_faces(
        &self,
        detections: &[FaceDetection],
        image: ndarray::ArrayView<f32, Ix4>,
    ) -> Result<Array<f32, Ix4>, AIProxyError> {
        if detections.is_empty() {
            return Ok(Array::zeros((0, 3, 112, 112)));
        }

        let mut cropped_faces: Vec<Array<f32, ndarray::Ix3>> = Vec::with_capacity(detections.len());

        for detection in detections.iter() {
            // Align each face to canonical pose using landmark-based transformation
            let aligned_face = self.align_face(detection, image)?;
            cropped_faces.push(aligned_face);
        }

        if cropped_faces.is_empty() {
            return Ok(Array::zeros((0, 3, 112, 112)));
        }

        // Stack all cropped faces into a batch
        // Convert Vec<Array3> to Array4
        let num_faces = cropped_faces.len();
        let channels = cropped_faces[0].shape()[0];
        let height = cropped_faces[0].shape()[1];
        let width = cropped_faces[0].shape()[2];

        let mut batch = Array::zeros((num_faces, channels, height, width));
        for (i, face) in cropped_faces.into_iter().enumerate() {
            batch.slice_mut(s![i, .., .., ..]).assign(&face);
        }

        Ok(batch)
    }

    /// Align face using 5 landmark points to canonical pose
    ///
    /// Face alignment is crucial for recognition accuracy. This method:
    /// 1. Maps detected landmark positions to standard reference positions (ArcFace protocol)
    /// 2. Estimates a similarity transform (scale, rotation, translation) between them
    /// 3. Warps the face to align it to the canonical pose
    ///
    /// Without alignment, the same person at different angles/scales would produce
    /// different embeddings. Alignment normalizes the pose for consistent recognition.
    #[tracing::instrument(skip(self, detection, image))]
    fn align_face(
        &self,
        detection: &FaceDetection,
        image: ndarray::ArrayView<f32, Ix4>,
    ) -> Result<Array<f32, Ix3>, AIProxyError> {
        // Target landmark positions for a properly aligned 112x112 face (ArcFace standard)
        // These positions define where eyes, nose, and mouth should be in the aligned output
        let reference_points = Array2::from_shape_vec(
            (5, 2),
            vec![
                30.2946 + 8.0,
                51.6963, // left eye
                65.5318 + 8.0,
                51.5014, // right eye
                48.0252 + 8.0,
                71.7366, // nose tip
                33.5493 + 8.0,
                92.3655, // left mouth corner
                62.7299 + 8.0,
                92.2041, // right mouth corner
            ],
        )
        .map_err(|e| AIProxyError::ModelProviderPreprocessingError(e.to_string()))?;

        // Actual landmark positions from detection (already in pixel coordinates)
        let source_points = Array2::from_shape_vec(
            (5, 2),
            vec![
                detection.landmarks[0][0],
                detection.landmarks[0][1], // left eye
                detection.landmarks[1][0],
                detection.landmarks[1][1], // right eye
                detection.landmarks[2][0],
                detection.landmarks[2][1], // nose
                detection.landmarks[3][0],
                detection.landmarks[3][1], // left mouth
                detection.landmarks[4][0],
                detection.landmarks[4][1], // right mouth
            ],
        )
        .map_err(|e| AIProxyError::ModelProviderPreprocessingError(e.to_string()))?;

        // Find transform that maps source → reference landmarks
        let transform = self.estimate_similarity_transform(&source_points, &reference_points)?;

        // Apply transform to warp the entire face region to canonical pose
        let warped_face = self.warp_affine(image, &transform, 112, 112)?;

        Ok(warped_face)
    }

    /// Estimate similarity transform (scale + rotation + translation) between point sets
    ///
    /// This is a simplified version of the Umeyama algorithm. It finds the best
    /// similarity transform that maps source points to destination points.
    ///
    /// A similarity transform preserves angles and ratios (unlike affine transforms
    /// which can shear/skew). This is appropriate for face alignment because faces
    /// don't deform - they only rotate, scale, and translate.
    ///
    /// Returns a 2x3 affine transformation matrix: [a b tx; c d ty]
    #[tracing::instrument(skip(self, src, dst))]
    fn estimate_similarity_transform(
        &self,
        src: &Array2<f32>,
        dst: &Array2<f32>,
    ) -> Result<Array2<f32>, AIProxyError> {
        // Simplified implementation using eye-based alignment
        // Full Umeyama would use SVD for optimal rotation estimation

        // Calculate centroids
        let src_mean = src.mean_axis(Axis(0)).unwrap();
        let dst_mean = dst.mean_axis(Axis(0)).unwrap();

        // Center the points
        let src_centered = src - &src_mean;
        let dst_centered = dst - &dst_mean;

        // Calculate scale
        let src_norm = src_centered.mapv(|x| x * x).sum().sqrt();
        let dst_norm = dst_centered.mapv(|x| x * x).sum().sqrt();
        let scale = dst_norm / src_norm;

        // Calculate rotation using mean eye positions (simplified)
        let src_eye_center_x = (src[[0, 0]] + src[[1, 0]]) / 2.0;
        let src_eye_center_y = (src[[0, 1]] + src[[1, 1]]) / 2.0;
        let dst_eye_center_x = (dst[[0, 0]] + dst[[1, 0]]) / 2.0;
        let dst_eye_center_y = (dst[[0, 1]] + dst[[1, 1]]) / 2.0;

        let src_dx = src[[1, 0]] - src[[0, 0]];
        let src_dy = src[[1, 1]] - src[[0, 1]];
        let dst_dx = dst[[1, 0]] - dst[[0, 0]];
        let dst_dy = dst[[1, 1]] - dst[[0, 1]];

        let src_angle = src_dy.atan2(src_dx);
        let dst_angle = dst_dy.atan2(dst_dx);
        let angle = dst_angle - src_angle;

        // Build affine transformation matrix [2x3]
        // [a  b  tx]
        // [c  d  ty]
        let cos_a = angle.cos() * scale;
        let sin_a = angle.sin() * scale;

        let tx = dst_eye_center_x - (src_eye_center_x * cos_a - src_eye_center_y * sin_a);
        let ty = dst_eye_center_y - (src_eye_center_x * sin_a + src_eye_center_y * cos_a);

        let transform =
            Array2::from_shape_vec((2, 3), vec![cos_a, -sin_a, tx, sin_a, cos_a, ty])
                .map_err(|e| AIProxyError::ModelProviderPreprocessingError(e.to_string()))?;

        Ok(transform)
    }

    /// Apply affine transformation to warp image using backward mapping
    ///
    /// Image warping is done using "backward mapping" (inverse transform):
    /// - For each pixel in the OUTPUT image, find where it came from in the SOURCE
    /// - Use bilinear interpolation to get smooth sub-pixel values
    ///
    /// Why backward mapping? Forward mapping (source→dest) leaves holes in the output
    /// because multiple source pixels might map to the same dest, or some dest pixels
    /// might have no source. Backward mapping guarantees every output pixel is filled.
    ///
    /// Bilinear interpolation: When source coordinates are fractional (e.g., 42.7),
    /// blend the 4 surrounding integer pixels for a smooth result.
    #[tracing::instrument(skip(self, image, transform))]
    fn warp_affine(
        &self,
        image: ndarray::ArrayView<f32, Ix4>,
        transform: &Array2<f32>,
        output_width: usize,
        output_height: usize,
    ) -> Result<Array<f32, Ix3>, AIProxyError> {
        let img_shape = image.shape();
        let channels = img_shape[1];
        let src_height = img_shape[2];
        let src_width = img_shape[3];

        // Memory check: Allocating transformed face image array
        // output_width/output_height: Typically 112×112 (ArcFace standard face size)
        // + 64: ndarray struct overhead
        let output_pixels = channels * output_height * output_width;
        let estimated_bytes = output_pixels * size_of::<f32>() + 64;
        utils::allocator::check_memory_available(estimated_bytes)
            .map_err(|e| AIProxyError::Allocation(e.into()))?;

        let mut output = Array::zeros((channels, output_height, output_width));

        // Compute inverse transform for backward mapping
        // Forward: dest = M * src, so src = M^-1 * dest
        let a = transform[[0, 0]];
        let b = transform[[0, 1]];
        let tx = transform[[0, 2]];
        let c = transform[[1, 0]];
        let d = transform[[1, 1]];
        let ty = transform[[1, 2]];

        let det = a * d - b * c;
        if det.abs() < 1e-6 {
            return Err(AIProxyError::ModelProviderPreprocessingError(
                "Singular transformation matrix".to_string(),
            ));
        }

        // Inverse 2x2 matrix components
        let inv_a = d / det;
        let inv_b = -b / det;
        let inv_c = -c / det;
        let inv_d = a / det;
        let inv_tx = (b * ty - d * tx) / det;
        let inv_ty = (c * tx - a * ty) / det;

        // For each output pixel, find corresponding source location
        for dst_y in 0..output_height {
            for dst_x in 0..output_width {
                let dst_xf = dst_x as f32;
                let dst_yf = dst_y as f32;

                // Apply inverse transform to find source coordinates
                let src_x = inv_a * dst_xf + inv_b * dst_yf + inv_tx;
                let src_y = inv_c * dst_xf + inv_d * dst_yf + inv_ty;

                // Bilinear interpolation
                if src_x >= 0.0
                    && src_x < (src_width - 1) as f32
                    && src_y >= 0.0
                    && src_y < (src_height - 1) as f32
                {
                    let x0 = src_x.floor() as usize;
                    let y0 = src_y.floor() as usize;
                    let x1 = x0 + 1;
                    let y1 = y0 + 1;

                    let dx = src_x - x0 as f32;
                    let dy = src_y - y0 as f32;

                    for ch in 0..channels {
                        let p00 = image[[0, ch, y0, x0]];
                        let p01 = image[[0, ch, y0, x1]];
                        let p10 = image[[0, ch, y1, x0]];
                        let p11 = image[[0, ch, y1, x1]];

                        let p0 = p00 * (1.0 - dx) + p01 * dx;
                        let p1 = p10 * (1.0 - dx) + p11 * dx;
                        let pixel = p0 * (1.0 - dy) + p1 * dy;

                        output[[ch, dst_y, dst_x]] = pixel;
                    }
                }
            }
        }

        Ok(output)
    }

    /// Run recognition model on cropped faces
    #[tracing::instrument(skip(self, faces, session))]
    fn recognize_faces(
        &self,
        faces: Array<f32, Ix4>,
        session: &Session,
    ) -> Result<Array<f32, Ix2>, AIProxyError> {
        // Recognition model expects "input.1" tensor (same as detection)
        let session_inputs = ort::inputs!["input.1" => faces.view()]
            .map_err(|e| AIProxyError::ModelProviderPreprocessingError(e.to_string()))?;

        let outputs = session
            .run(session_inputs)
            .map_err(|e| AIProxyError::ModelProviderRunInferenceError(e.to_string()))?;

        // Extract embeddings from output
        // ResNet50 outputs a tensor with embeddings
        let embeddings = outputs[0]
            .try_extract_tensor::<f32>()
            .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;

        let shape = embeddings.shape();
        let embedding_array = embeddings
            .to_owned()
            .into_shape_with_order((shape[0], shape[1]))
            .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;

        Ok(embedding_array)
    }
}

/// Represents a detected face with bounding box, landmarks, and confidence
///
/// RetinaFace detects faces and provides:
/// - Bounding box: Face location in [x1, y1, x2, y2] format (pixel coordinates)
/// - Landmarks: 5 facial keypoints for alignment (left eye, right eye, nose, mouth corners)
/// - Confidence: Detection score (higher = more confident)
#[derive(Debug, Clone)]
struct FaceDetection {
    bbox: [f32; 4],           // [x1, y1, x2, y2] in pixels
    landmarks: [[f32; 2]; 5], // [[x, y]; 5] - left_eye, right_eye, nose, left_mouth, right_mouth
    confidence: f32,          // Detection confidence score (0.0 to 1.0)
}
