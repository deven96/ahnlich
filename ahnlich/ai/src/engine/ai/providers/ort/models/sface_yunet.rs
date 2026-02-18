use super::super::InnerAIExecutionProvider;
use super::super::executor::ExecutorWithSessionCache;
use super::super::inference_model::ORTInferenceModel;
use super::face_align::{FaceDetection, apply_nms, crop_and_align_faces};
use crate::engine::ai::models::{ModelInput, ModelResponse};
use crate::error::AIProxyError;
use ahnlich_types::ai::execution_provider::ExecutionProvider as AIExecutionProvider;
use ahnlich_types::keyval::StoreKey;
use hf_hub::api::sync::Api;
use ndarray::{Array, Axis, Ix2, Ix3, Ix4, concatenate, s};
use ort::Session;
use std::future::Future;
use std::mem::size_of;
use std::pin::Pin;

/// SFace + YuNet: Commercially-licensed multi-stage face detection + recognition pipeline
///
/// Uses two models from OpenCV Zoo, both available under permissive open-source licenses:
/// - YuNet (MIT):     Lightweight face detector producing 5-landmark detections
/// - SFace (Apache 2.0): MobileFaceNet trained with sigmoid-constrained hypersphere loss
///
/// Pipeline: Image → YuNet detection → 5-landmark crop/align → SFace recognition → 128-dim embeddings
/// Mode: OneToMany — returns one 128-dimensional embedding per detected face
///
/// HuggingFace sources:
///   https://huggingface.co/deven96/face_detection_yunet
///   https://huggingface.co/deven96/face_recognition_sface
pub(crate) struct SfaceYunetModel {
    detection_cache: ExecutorWithSessionCache, // YuNet face detector
    recognition_cache: ExecutorWithSessionCache, // SFace face recognizer
    model_batch_size: usize,
}

impl SfaceYunetModel {
    #[tracing::instrument(skip_all)]
    pub async fn build(
        api: Api,
        session_profiling: bool,
    ) -> Result<Box<dyn ORTInferenceModel>, AIProxyError> {
        let det_repo = api.model("deven96/face_detection_yunet".to_string());
        let det_file = det_repo
            .get("face_detection_yunet_2023mar.onnx")
            .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?;
        let det_cache = ExecutorWithSessionCache::new(det_file, session_profiling);
        det_cache
            .try_get_with(InnerAIExecutionProvider::CPU)
            .await?;

        let rec_repo = api.model("deven96/face_recognition_sface".to_string());
        let rec_file = rec_repo
            .get("face_recognition_sface_2021dec.onnx")
            .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?;
        let rec_cache = ExecutorWithSessionCache::new(rec_file, session_profiling);
        rec_cache
            .try_get_with(InnerAIExecutionProvider::CPU)
            .await?;

        Ok(Box::new(Self {
            detection_cache: det_cache,
            recognition_cache: rec_cache,
            model_batch_size: 16,
        }))
    }
}

impl ORTInferenceModel for SfaceYunetModel {
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
                    model_name: "SFace+YuNet (image-only model)".to_string(),
                }),
            }
        })
    }

    fn batch_size(&self) -> usize {
        self.model_batch_size
    }
}

impl SfaceYunetModel {
    /// Main pipeline: detect faces with YuNet, then recognize all faces with SFace in one pass
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

        // Stage 1: Detect and align faces from each image using YuNet
        for image_idx in 0..batch_size {
            let single_image = images.slice(s![image_idx..image_idx + 1, .., .., ..]);
            let detections = self.detect_faces(single_image.to_owned(), &det_session)?;

            if detections.is_empty() {
                face_counts.push(0);
                continue;
            }

            let cropped_faces = crop_and_align_faces(&detections, single_image)?;
            let num_faces = cropped_faces.shape()[0];
            face_counts.push(num_faces);

            for face_idx in 0..num_faces {
                all_cropped_faces.push(cropped_faces.slice(s![face_idx, .., .., ..]).to_owned());
            }
        }

        // Stage 2: Batch recognize all detected faces in one SFace forward pass
        let embeddings = if all_cropped_faces.is_empty() {
            Array::zeros((0, 128))
        } else {
            // SFace expects 112×112 aligned faces — same ArcFace-standard crop as Buffalo_L
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

        // Stage 3: Map 128-dim embeddings back to source images
        let total_faces: usize = face_counts.iter().sum();
        let bytes_per_face =
            size_of::<ModelResponse>() + size_of::<StoreKey>() + (128 * size_of::<f32>()) + 64;
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

    /// Run YuNet detection on a single image (shape: 1×C×H×W)
    ///
    /// YuNet outputs 12 tensors across 3 FPN strides (8, 16, 32):
    ///   cls_8/16/32  — classification scores  (1, N, 1)
    ///   obj_8/16/32  — objectness scores       (1, N, 1)
    ///   bbox_8/16/32 — bbox offsets            (1, N, 4)  [cx, cy, w, h] in stride units
    ///   kps_8/16/32  — keypoint offsets        (1, N, 10) [5 × (dx, dy)] in stride units
    ///
    /// Decoded score = sigmoid(cls) * sigmoid(obj). Boxes decoded from anchor center + offset.
    ///
    /// Input image arrives as RGB 0-255. YuNet was trained with OpenCV's blobFromImage which
    /// produces BGR 0-255, so we swap channels R↔B before inference.
    #[tracing::instrument(skip(self, image, session))]
    fn detect_faces(
        &self,
        image: Array<f32, Ix4>,
        session: &Session,
    ) -> Result<Vec<FaceDetection>, AIProxyError> {
        let img_h = image.shape()[2] as f32;
        let img_w = image.shape()[3] as f32;

        // Swap RGB → BGR: channel 0 (R) ↔ channel 2 (B), channel 1 (G) stays.
        // YuNet was trained with OpenCV blobFromImage which produces BGR 0-255.
        let r = image.slice(s![.., 0..1, .., ..]).to_owned();
        let g = image.slice(s![.., 1..2, .., ..]).to_owned();
        let b = image.slice(s![.., 2..3, .., ..]).to_owned();
        let bgr_image = concatenate(Axis(1), &[b.view(), g.view(), r.view()])
            .map_err(|e| AIProxyError::ModelProviderPreprocessingError(e.to_string()))?;

        let session_inputs = ort::inputs!["input" => bgr_image.view()]
            .map_err(|e| AIProxyError::ModelProviderPreprocessingError(e.to_string()))?;

        let outputs = session
            .run(session_inputs)
            .map_err(|e| AIProxyError::ModelProviderRunInferenceError(e.to_string()))?;

        const CONFIDENCE_THRESHOLD: f32 = 0.6;

        // Extract all 12 tensors
        let cls_8 = outputs["cls_8"]
            .try_extract_tensor::<f32>()
            .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;
        let cls_16 = outputs["cls_16"]
            .try_extract_tensor::<f32>()
            .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;
        let cls_32 = outputs["cls_32"]
            .try_extract_tensor::<f32>()
            .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;
        let obj_8 = outputs["obj_8"]
            .try_extract_tensor::<f32>()
            .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;
        let obj_16 = outputs["obj_16"]
            .try_extract_tensor::<f32>()
            .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;
        let obj_32 = outputs["obj_32"]
            .try_extract_tensor::<f32>()
            .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;
        let bbox_8 = outputs["bbox_8"]
            .try_extract_tensor::<f32>()
            .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;
        let bbox_16 = outputs["bbox_16"]
            .try_extract_tensor::<f32>()
            .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;
        let bbox_32 = outputs["bbox_32"]
            .try_extract_tensor::<f32>()
            .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;
        let kps_8 = outputs["kps_8"]
            .try_extract_tensor::<f32>()
            .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;
        let kps_16 = outputs["kps_16"]
            .try_extract_tensor::<f32>()
            .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;
        let kps_32 = outputs["kps_32"]
            .try_extract_tensor::<f32>()
            .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;

        let mut detections = Vec::new();

        // Process each stride scale
        for (stride, cls, obj, bbox, kps) in [
            (8usize, &cls_8, &obj_8, &bbox_8, &kps_8),
            (16, &cls_16, &obj_16, &bbox_16, &kps_16),
            (32, &cls_32, &obj_32, &bbox_32, &kps_32),
        ] {
            let stride_f = stride as f32;
            let feat_w = (img_w / stride_f) as usize;
            let n = cls.shape()[1]; // number of anchors at this scale

            for i in 0..n {
                // cls and obj outputs are already sigmoid-activated probabilities in [0,1]
                let cls_score = cls[[0, i, 0]];
                let obj_score = obj[[0, i, 0]];
                let score = cls_score * obj_score;

                if score < CONFIDENCE_THRESHOLD {
                    continue;
                }

                // Anchor center: grid position (col, row) in feature map → pixel coords
                let row = i / feat_w;
                let col = i % feat_w;
                let anchor_cx = (col as f32 + 0.5) * stride_f;
                let anchor_cy = (row as f32 + 0.5) * stride_f;

                // Decode bbox — offsets are in stride units, box is [cx_off, cy_off, w, h]
                let cx = anchor_cx + bbox[[0, i, 0]] * stride_f;
                let cy = anchor_cy + bbox[[0, i, 1]] * stride_f;
                let w = bbox[[0, i, 2]] * stride_f;
                let h = bbox[[0, i, 3]] * stride_f;

                let x1 = (cx - w / 2.0).clamp(0.0, img_w);
                let y1 = (cy - h / 2.0).clamp(0.0, img_h);
                let x2 = (cx + w / 2.0).clamp(0.0, img_w);
                let y2 = (cy + h / 2.0).clamp(0.0, img_h);

                // Decode 5 landmarks — offsets relative to anchor center, in stride units
                let landmarks = [
                    [
                        anchor_cx + kps[[0, i, 0]] * stride_f,
                        anchor_cy + kps[[0, i, 1]] * stride_f,
                    ],
                    [
                        anchor_cx + kps[[0, i, 2]] * stride_f,
                        anchor_cy + kps[[0, i, 3]] * stride_f,
                    ],
                    [
                        anchor_cx + kps[[0, i, 4]] * stride_f,
                        anchor_cy + kps[[0, i, 5]] * stride_f,
                    ],
                    [
                        anchor_cx + kps[[0, i, 6]] * stride_f,
                        anchor_cy + kps[[0, i, 7]] * stride_f,
                    ],
                    [
                        anchor_cx + kps[[0, i, 8]] * stride_f,
                        anchor_cy + kps[[0, i, 9]] * stride_f,
                    ],
                ];

                detections.push(FaceDetection {
                    bbox: [x1, y1, x2, y2],
                    landmarks,
                    confidence: score,
                });
            }
        }

        Ok(apply_nms(detections, 0.4))
    }

    /// Run SFace recognition on a batch of aligned 112×112 face crops
    ///
    /// SFace has a fixed batch size of 1, so we run inference one face at a time
    /// and stack the resulting embeddings into a (n_faces, 128) array.
    ///
    /// SFace input:  "data" (1, 3, 112, 112)
    /// SFace output: "fc1"  (1, 128) — L2-normalised face embeddings
    #[tracing::instrument(skip(self, faces, session))]
    fn recognize_faces(
        &self,
        faces: Array<f32, Ix4>,
        session: &Session,
    ) -> Result<Array<f32, Ix2>, AIProxyError> {
        let n_faces = faces.shape()[0];
        let mut all_embeddings: Vec<Array<f32, Ix2>> = Vec::with_capacity(n_faces);

        for i in 0..n_faces {
            let single_face = faces.slice(s![i..i + 1, .., .., ..]).to_owned();

            let session_inputs = ort::inputs!["data" => single_face.view()]
                .map_err(|e| AIProxyError::ModelProviderPreprocessingError(e.to_string()))?;

            let outputs = session
                .run(session_inputs)
                .map_err(|e| AIProxyError::ModelProviderRunInferenceError(e.to_string()))?;

            // Output tensor is named "fc1" — (1, 128) L2-normalised embedding
            let embedding = outputs["fc1"]
                .try_extract_tensor::<f32>()
                .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;

            let emb_shape = embedding.shape();
            let emb_2d = embedding
                .to_owned()
                .into_shape_with_order((emb_shape[0], emb_shape[1]))
                .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;

            all_embeddings.push(emb_2d);
        }

        // Stack all (1, 128) embeddings into (n_faces, 128)
        let embedding_views: Vec<_> = all_embeddings.iter().map(|e| e.view()).collect();
        ndarray::concatenate(Axis(0), &embedding_views)
            .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))
    }
}
