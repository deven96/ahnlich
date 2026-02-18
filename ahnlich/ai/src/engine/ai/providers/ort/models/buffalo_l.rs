use super::super::InnerAIExecutionProvider;
use super::super::executor::ExecutorWithSessionCache;
use super::super::inference_model::ORTInferenceModel;
use super::face_align::{FaceDetection, apply_nms, crop_and_align_faces};
use crate::engine::ai::models::{ModelInput, ModelResponse};
use crate::error::AIProxyError;
use ahnlich_types::ai::execution_provider::ExecutionProvider as AIExecutionProvider;
use ahnlich_types::keyval::StoreKey;
use hf_hub::api::sync::Api;
use ndarray::{Array, Axis, Ix2, Ix3, Ix4, s};
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
        let repo = api.model("deven96/buffalo_l".to_string());

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
            let cropped_faces = crop_and_align_faces(&detections, single_image)?;
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
        Ok(apply_nms(all_detections, 0.4))
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
