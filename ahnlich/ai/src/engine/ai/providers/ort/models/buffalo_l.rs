use super::super::InnerAIExecutionProvider;
use super::super::executor::ExecutorWithSessionCache;
use super::super::inference_model::ORTInferenceModel;
use super::bbox_utils::apply_letterbox_correction;
use super::face::align::{FaceDetection, apply_nms, crop_and_align_from_original};
use super::face::{detect, recognize};
use crate::engine::ai::models::{ImageBatch, ModelInput, ModelResponse};
use crate::error::AIProxyError;
use ahnlich_types::ai::execution_provider::ExecutionProvider as AIExecutionProvider;
use ahnlich_types::keyval::StoreKey;
use ahnlich_types::metadata::MetadataValue;
use hf_hub::api::sync::Api;
use ndarray::{Array, Axis, Ix3, Ix4, s};
use ort::Session;
use rayon::prelude::*;
use std::collections::HashMap;
use std::future::Future;
use std::mem::size_of;
use std::pin::Pin;

/// SCRFD detection, then ArcFace recognition on a 112x112 crop aligned from the detected
/// landmarks: one embedding per face (OneToMany). Gender/age is off unless asked for.
///
/// A change here holds only if the same person still scores > 0.5 cosine and two different
/// people < 0.3 (see `buffalo_l_test.rs`); `cargo test -p ai --lib buffalo_l` checks it.
pub(crate) struct BuffaloLModel {
    detection_cache: ExecutorWithSessionCache,
    recognition_cache: ExecutorWithSessionCache,
    genderage_cache: ExecutorWithSessionCache,
    model_batch_size: usize,
    anchors: Vec<detect::Anchor>,
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

        // Load gender/age model
        let genderage_file = repo
            .get("genderage/model.onnx")
            .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?;
        let genderage_cache = ExecutorWithSessionCache::new(genderage_file, session_profiling);
        genderage_cache
            .try_get_with(InnerAIExecutionProvider::CPU)
            .await?;

        tracing::info!("Loaded gender/age model for BuffaloL");

        let anchors = detect::generate_anchors();

        Ok(Box::new(Self {
            detection_cache: det_cache,
            recognition_cache: rec_cache,
            genderage_cache,
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
        model_params: &std::collections::HashMap<String, String>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ModelResponse>, AIProxyError>> + Send + '_>> {
        let model_params = model_params.clone();
        Box::pin(async move {
            match input {
                ModelInput::Images(images) => {
                    self.detect_and_recognize_batch(images, execution_provider, &model_params)
                        .await
                }
                ModelInput::Texts(_) | ModelInput::Audios(_) => {
                    Err(AIProxyError::AIModelNotSupported {
                        model_name: "Buffalo_L (image-only model)".to_string(),
                    })
                }
            }
        })
    }

    fn batch_size(&self) -> usize {
        self.model_batch_size
    }
}

impl BuffaloLModel {
    /// Detection is per image; recognition is one batched pass over every face found.
    #[tracing::instrument(skip(self, batch))]
    async fn detect_and_recognize_batch(
        &self,
        batch: ImageBatch,
        execution_provider: Option<AIExecutionProvider>,
        model_params: &std::collections::HashMap<String, String>,
    ) -> Result<Vec<ModelResponse>, AIProxyError> {
        // Parse attributes parameter to check if genderage prediction is requested
        let should_predict_genderage = model_params
            .get("attributes")
            .map(|attrs| attrs.split(',').any(|attr| attr.trim() == "genderage"))
            .unwrap_or(false);

        let exec_prov = execution_provider
            .map(|ep| ep.into())
            .unwrap_or(InnerAIExecutionProvider::CPU);
        let det_session = self.detection_cache.try_get_with(exec_prov).await?;
        let rec_session = self.recognition_cache.try_get_with(exec_prov).await?;

        let ImageBatch {
            tensor: images,
            originals,
        } = batch;

        let batch_size = images.shape()[0];
        let mut all_cropped_faces: Vec<Array<f32, Ix3>> = Vec::new();
        let mut all_detections: Vec<FaceDetection> = Vec::new();
        let mut face_counts: Vec<usize> = Vec::with_capacity(batch_size);

        // Stage 1: Detect and extract faces from each input image
        for image_idx in 0..batch_size {
            let single_image = images.slice(s![image_idx..image_idx + 1, .., .., ..]);
            let detections =
                self.detect_faces(single_image.to_owned(), &det_session, model_params)?;

            if detections.is_empty() {
                face_counts.push(0);
                continue;
            }

            // Cropping from the tensor instead would produce an embedding that is not
            // comparable with the rest of the store, so a missing original is an error.
            let original = originals.get(image_idx).ok_or_else(|| {
                AIProxyError::ModelProviderPreprocessingError(format!(
                    "image {image_idx} reached recognition without its original pixels"
                ))
            })?;

            let aligned = crop_and_align_from_original(
                &detections,
                original.image(),
                detect::INPUT_SIZE as u32,
            );

            face_counts.push(aligned.len());

            for (face_idx, face) in aligned.faces.into_iter().enumerate() {
                all_cropped_faces.push(aligned.crops.slice(s![face_idx, .., .., ..]).to_owned());
                all_detections.push(face);
            }
        }

        // Stages 2 & 3: Run embedding extraction and gender/age prediction in parallel
        // Both operations are independent and only depend on Stage 1 (face detection)
        let exec_prov = execution_provider
            .map(|ep| ep.into())
            .unwrap_or(InnerAIExecutionProvider::CPU);

        // Stage 2: Batch recognize all detected faces in one forward pass
        let recognize_stage = async {
            if all_cropped_faces.is_empty() {
                Ok(Array::zeros((0, 512)))
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

                recognize::embed(faces_batch, &rec_session)
            }
        };

        // Stage 3: Gender/Age prediction (optional, controlled by attributes parameter)
        let (embeddings, gender_age_attrs_opt) = if should_predict_genderage {
            // Run both recognition and genderage stages in parallel
            let genderage_stage = async {
                let genderage_session = self.genderage_cache.try_get_with(exec_prov).await?;
                Self::predict_gender_age(&genderage_session, &images, &all_detections).await
            };

            let (emb, gender_age) = tokio::join!(recognize_stage, genderage_stage);
            (emb?, Some(gender_age?))
        } else {
            // Run only recognition stage
            (recognize_stage.await?, None)
        };

        let embeddings = embeddings;
        let gender_age_attrs = gender_age_attrs_opt;

        // Step 4: Map embeddings back to source images with bounding box metadata
        // Memory check: Building ModelResponse for each face embedding
        // 512: the ArcFace embedding dimension.
        // + 64: Vec overhead for StoreKey
        let total_faces: usize = face_counts.iter().sum();
        let bytes_per_face =
            size_of::<ModelResponse>() + size_of::<StoreKey>() + (512 * size_of::<f32>()) + 64;
        let estimated_bytes = total_faces * bytes_per_face;
        utils::allocator::check_memory_available(estimated_bytes)
            .map_err(|e| AIProxyError::Allocation(e.into()))?;

        let mut results: Vec<ModelResponse> = Vec::with_capacity(batch_size);
        let mut embedding_offset = 0;

        for (image_idx, &num_faces) in face_counts.iter().enumerate() {
            if num_faces == 0 {
                results.push(ModelResponse::OneToMany(vec![]));
            } else {
                let face_keys_with_metadata: Vec<(
                    StoreKey,
                    Option<HashMap<String, MetadataValue>>,
                )> = embeddings
                    .slice(s![embedding_offset..embedding_offset + num_faces, ..])
                    .axis_iter(Axis(0))
                    .enumerate()
                    .map(|(idx, embedding)| {
                        let detection = &all_detections[embedding_offset + idx];
                        let mut metadata = HashMap::new();

                        // Per image: the handler injects orig_width_{i} for each one.
                        let dimension = |name: &str| {
                            model_params
                                .get(&format!("{name}_{image_idx}"))
                                .and_then(|s| s.parse::<f32>().ok())
                                .unwrap_or(detect::INPUT_SIZE as f32)
                        };
                        let orig_width = dimension("orig_width");
                        let orig_height = dimension("orig_height");

                        // Apply letterbox correction and normalize bounding box to 0-1 range
                        let normalized_bbox = apply_letterbox_correction(
                            &detection.bbox,
                            orig_width,
                            orig_height,
                            detect::INPUT_SIZE as f32,
                        );

                        // Store normalized bounding box coordinates (0-1 range)
                        metadata.insert(
                            "bbox_x1".to_string(),
                            MetadataValue {
                                value: Some(
                                    ahnlich_types::metadata::metadata_value::Value::RawString(
                                        normalized_bbox.x1.to_string(),
                                    ),
                                ),
                            },
                        );
                        metadata.insert(
                            "bbox_y1".to_string(),
                            MetadataValue {
                                value: Some(
                                    ahnlich_types::metadata::metadata_value::Value::RawString(
                                        normalized_bbox.y1.to_string(),
                                    ),
                                ),
                            },
                        );
                        metadata.insert(
                            "bbox_x2".to_string(),
                            MetadataValue {
                                value: Some(
                                    ahnlich_types::metadata::metadata_value::Value::RawString(
                                        normalized_bbox.x2.to_string(),
                                    ),
                                ),
                            },
                        );
                        metadata.insert(
                            "bbox_y2".to_string(),
                            MetadataValue {
                                value: Some(
                                    ahnlich_types::metadata::metadata_value::Value::RawString(
                                        normalized_bbox.y2.to_string(),
                                    ),
                                ),
                            },
                        );
                        metadata.insert(
                            "confidence".to_string(),
                            MetadataValue {
                                value: Some(
                                    ahnlich_types::metadata::metadata_value::Value::RawString(
                                        detection.confidence.to_string(),
                                    ),
                                ),
                            },
                        );

                        // Conditionally include gender/age (only when attributes=genderage was requested)
                        if let Some(ref genderage_vec) = gender_age_attrs
                            && let Some(&(female_prob, male_prob, age)) =
                                genderage_vec.get(embedding_offset + idx)
                        {
                            // Gender probabilities
                            metadata.insert(
                                "gender_female_prob".to_string(),
                                MetadataValue {
                                    value: Some(
                                        ahnlich_types::metadata::metadata_value::Value::RawString(
                                            female_prob.to_string(),
                                        ),
                                    ),
                                },
                            );
                            metadata.insert(
                                "gender_male_prob".to_string(),
                                MetadataValue {
                                    value: Some(
                                        ahnlich_types::metadata::metadata_value::Value::RawString(
                                            male_prob.to_string(),
                                        ),
                                    ),
                                },
                            );

                            // Age
                            metadata.insert(
                                "age".to_string(),
                                MetadataValue {
                                    value: Some(
                                        ahnlich_types::metadata::metadata_value::Value::RawString(
                                            age.to_string(),
                                        ),
                                    ),
                                },
                            );
                        }

                        (
                            StoreKey {
                                key: embedding.to_vec(),
                            },
                            Some(metadata),
                        )
                    })
                    .collect();

                results.push(ModelResponse::OneToMany(face_keys_with_metadata));
                embedding_offset += num_faces;
            }
        }

        Ok(results)
    }

    /// Run gender/age prediction on a batch of aligned face crops
    ///
    /// Input: Batch of aligned face images (112x112 from recognition pipeline)
    /// Output: Vec of (gender_female_prob, gender_male_prob, age) tuples
    /// Extract gender and age attributes using bbox-based affine transformation.
    /// Follows InsightFace's approach: scale = input_size / (max(w,h) * 1.5)
    async fn predict_gender_age(
        genderage_session: &Session,
        letterboxed_images: &Array<f32, Ix4>,
        detections: &[FaceDetection],
    ) -> Result<Vec<(f32, f32, i32)>, AIProxyError> {
        if detections.is_empty() {
            return Ok(vec![]);
        }

        const INPUT_SIZE: usize = 96;
        let (_, _, img_h, img_w) = letterboxed_images.dim();

        // Parallelize crop extraction and preprocessing using rayon
        // This is the CPU-intensive part (resizing, denormalization)
        let preprocessed_crops: Result<Vec<_>, AIProxyError> = detections
            .par_iter()
            .map(|detection| {
                let bbox = &detection.bbox;
                let bbox_w = bbox[2] - bbox[0];
                let bbox_h = bbox[3] - bbox[1];
                let center_x = (bbox[0] + bbox[2]) / 2.0;
                let center_y = (bbox[1] + bbox[3]) / 2.0;

                let scale = INPUT_SIZE as f32 / (bbox_w.max(bbox_h) * 1.5);
                let src_size = INPUT_SIZE as f32 / scale;

                let src_x1 = ((center_x - src_size / 2.0).max(0.0) as usize).min(img_w);
                let src_y1 = ((center_y - src_size / 2.0).max(0.0) as usize).min(img_h);
                let src_x2 = ((src_x1 as f32 + src_size).min(img_w as f32) as usize).min(img_w);
                let src_y2 = ((src_y1 as f32 + src_size).min(img_h as f32) as usize).min(img_h);

                let crop_w = (src_x2 - src_x1).max(1);
                let crop_h = (src_y2 - src_y1).max(1);

                // Extract and resize crop to 96x96
                let cropped = letterboxed_images
                    .slice(s![0, .., src_y1..src_y2, src_x1..src_x2])
                    .to_owned();
                let mut resized = Array::zeros((3, INPUT_SIZE, INPUT_SIZE));

                for (y, x) in (0..INPUT_SIZE).flat_map(|y| (0..INPUT_SIZE).map(move |x| (y, x))) {
                    let src_y = ((y as f32 * crop_h as f32 / INPUT_SIZE as f32).floor() as usize)
                        .min(crop_h - 1);
                    let src_x = ((x as f32 * crop_w as f32 / INPUT_SIZE as f32).floor() as usize)
                        .min(crop_w - 1);
                    for c in 0..3 {
                        resized[[c, y, x]] = cropped[[c, src_y, src_x]];
                    }
                }

                // Denormalize [-1,1] -> [0,255] (model expects raw pixels)
                Ok(resized.mapv(|v| (v + 1.0) * 127.5).insert_axis(Axis(0)))
            })
            .collect();

        let preprocessed_crops = preprocessed_crops?;

        // Run inference sequentially (ONNX sessions are not thread-safe)
        let mut results = Vec::with_capacity(detections.len());
        for input in preprocessed_crops.into_iter() {
            let outputs =
                genderage_session
                    .run(ort::inputs!["data" => input.view()].map_err(|e| {
                        AIProxyError::ModelProviderPreprocessingError(e.to_string())
                    })?)
                    .map_err(|e| AIProxyError::ModelProviderRunInferenceError(e.to_string()))?;

            let combined = outputs[0]
                .try_extract_tensor::<f32>()
                .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;

            if combined.shape() != [1, 3] {
                return Err(AIProxyError::ModelProviderPostprocessingError(format!(
                    "Expected output shape [1, 3], got {:?}",
                    combined.shape()
                )));
            }

            let female_logit = combined.view()[[0, 0]];
            let male_logit = combined.view()[[0, 1]];
            let age_raw = combined.view()[[0, 2]];

            let exp_female = female_logit.exp();
            let exp_male = male_logit.exp();
            let female_prob = exp_female / (exp_female + exp_male);
            let male_prob = exp_male / (exp_female + exp_male);
            let age = (age_raw * 100.0).round() as i32;

            results.push((female_prob, male_prob, age));
        }

        Ok(results)
    }

    #[tracing::instrument(skip(self, image, session))]
    fn detect_faces(
        &self,
        image: Array<f32, Ix4>,
        session: &Session,
        model_params: &std::collections::HashMap<String, String>,
    ) -> Result<Vec<FaceDetection>, AIProxyError> {
        let session_inputs = ort::inputs!["input.1" => image.view()]
            .map_err(|e| AIProxyError::ModelProviderPreprocessingError(e.to_string()))?;

        let outputs = session
            .run(session_inputs)
            .map_err(|e| AIProxyError::ModelProviderRunInferenceError(e.to_string()))?;

        self.parse_detections(outputs, model_params)
    }

    /// Each face fires on several anchors across the pyramid levels; NMS collapses them.
    #[tracing::instrument(skip(self, outputs, model_params))]
    fn parse_detections(
        &self,
        outputs: ort::SessionOutputs,
        model_params: &std::collections::HashMap<String, String>,
    ) -> Result<Vec<FaceDetection>, AIProxyError> {
        const DEFAULT_CONFIDENCE_THRESHOLD: f32 = 0.5;
        // Group photos need a low IoU: neighbouring faces overlap and a high threshold
        // merges them into one.
        const DEFAULT_NMS_THRESHOLD: f32 = 0.4;

        let param = |name: &str, fallback: f32| -> f32 {
            model_params
                .get(name)
                .and_then(|v| v.parse().ok())
                .unwrap_or(fallback)
        };
        let confidence_threshold = param("confidence_threshold", DEFAULT_CONFIDENCE_THRESHOLD);
        let nms_threshold = param("nms_threshold", DEFAULT_NMS_THRESHOLD);

        // 50 faces is a generous ceiling for one image, plus Vec overhead.
        utils::allocator::check_memory_available(50 * size_of::<FaceDetection>() + 64)
            .map_err(|e| AIProxyError::Allocation(e.into()))?;

        let tensors: Vec<_> = outputs
            .values()
            .map(|value| value.try_extract_tensor::<f32>())
            .collect::<Result<Vec<_>, _>>()
            .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;

        let faces = detect::decode(&tensors, &self.anchors, confidence_threshold)?;

        Ok(apply_nms(faces, nms_threshold))
    }
}
