use super::SupportedModels;
use super::executor::ExecutorWithSessionCache;
use super::inference_model::{ORTInferenceModel, ORTModality};
use crate::engine::ai::models::{ModelInput, ModelResponse};
use crate::error::AIProxyError;
use ahnlich_types::ai::execution_provider::ExecutionProvider as AIExecutionProvider;
use ahnlich_types::keyval::StoreKey;
use fallible_collections::FallibleVec;
use itertools::Itertools;
use ndarray::{Array, ArrayView1, Axis, Ix2, Ix4};
use ort::{Session, Value};
use rayon::prelude::*;
use std::future::Future;
use std::mem::size_of;
use std::pin::Pin;
use tokenizers::Encoding;
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub(crate) struct SingleStageModel {
    pub(super) model_type: ORTModality,
    pub(super) executor_session_cache: ExecutorWithSessionCache,
    pub(super) model_batch_size: usize,
    pub(super) supported_models: SupportedModels,
}

impl SingleStageModel {
    pub fn new(
        model_type: ORTModality,
        executor_session_cache: ExecutorWithSessionCache,
        model_batch_size: usize,
        supported_models: SupportedModels,
    ) -> Self {
        Self {
            model_type,
            executor_session_cache,
            model_batch_size,
            supported_models,
        }
    }

    #[tracing::instrument(skip(self, inputs, session))]
    fn batch_inference_image(
        &self,
        inputs: Array<f32, Ix4>,
        session: &Session,
    ) -> Result<Array<f32, Ix2>, AIProxyError> {
        let input_param = match self.supported_models {
            SupportedModels::Resnet50 => "input",
            SupportedModels::ClipVitB32Image => "pixel_values",
            _ => {
                return Err(AIProxyError::AIModelNotSupported {
                    model_name: self.supported_models.to_string(),
                });
            }
        };

        let session_inputs = ort::inputs![
            input_param => inputs.view(),
        ]
        .map_err(|e| AIProxyError::ModelProviderPreprocessingError(e.to_string()))?;

        let child_span = tracing::info_span!("image-model-session-run");
        child_span.set_parent(Span::current().context());
        let child_guard = child_span.enter();
        let outputs = session
            .run(session_inputs)
            .map_err(|e| AIProxyError::ModelProviderRunInferenceError(e.to_string()))?;
        drop(child_guard);

        // Postprocess output directly here
        let output_tensor = outputs
            .values()
            .next()
            .ok_or_else(|| {
                AIProxyError::ModelProviderPostprocessingError("No output tensor found".to_string())
            })?
            .try_extract_tensor::<f32>()
            .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;

        let mut embeddings = output_tensor
            .to_owned()
            .into_dimensionality::<Ix2>()
            .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;

        // Apply normalization if needed (Resnet50 and Buffalo_L need it)
        if matches!(
            self.supported_models,
            SupportedModels::Resnet50 | SupportedModels::BuffaloL
        ) {
            self.normalize_embeddings(&mut embeddings);
        }

        Ok(embeddings)
    }

    /// Normalize embeddings to unit vectors (L2 normalization)
    fn normalize_embeddings(&self, embeddings: &mut Array<f32, Ix2>) {
        use ndarray::Axis;
        for mut row in embeddings.axis_iter_mut(Axis(0)) {
            let norm: f32 = row.iter().map(|x| x * x).sum::<f32>().sqrt();
            if norm > 0.0 {
                row.mapv_inplace(|x| x / norm);
            }
        }
    }

    #[tracing::instrument(skip(self, encodings, session))]
    fn batch_inference_text(
        &self,
        encodings: Vec<Encoding>,
        session: &Session,
    ) -> Result<Array<f32, Ix2>, AIProxyError> {
        if self.model_type != ORTModality::Text {
            return Err(AIProxyError::AIModelNotSupported {
                model_name: self.supported_models.to_string(),
            });
        }
        let batch_size = encodings.len();
        // Extract the encoding length and batch size
        let encoding_length = encodings[0].len();
        let max_size = encoding_length * batch_size;

        let need_token_type_ids = session
            .inputs
            .iter()
            .any(|input| input.name == "token_type_ids");

        // Memory check: Allocating Vec<i64> for tokenized text inputs
        // 2 or 3 arrays: input_ids, attention_mask, and token_type_ids (if model requires it)
        let num_arrays = if need_token_type_ids { 3 } else { 2 };
        let estimated_bytes = max_size * size_of::<i64>() * num_arrays;
        utils::allocator::check_memory_available(estimated_bytes)
            .map_err(|e| AIProxyError::Allocation(e.into()))?;
        let mut ids_array = Vec::with_capacity(max_size);
        let mut mask_array = Vec::with_capacity(max_size);
        let mut token_type_ids_array: Option<Vec<i64>> = None;
        if need_token_type_ids {
            token_type_ids_array = Some(Vec::with_capacity(max_size));
        }

        // Not using par_iter because the closure needs to be FnMut
        encodings.iter().for_each(|encoding| {
            let ids = encoding.get_ids();
            let mask = encoding.get_attention_mask();

            // Extend the preallocated arrays with the current encoding
            // Requires the closure to be FnMut
            ids_array.extend(ids.iter().map(|x| *x as i64));
            mask_array.extend(mask.iter().map(|x| *x as i64));
            if let Some(ref mut token_type_ids_array) = token_type_ids_array {
                token_type_ids_array.extend(encoding.get_type_ids().iter().map(|x| *x as i64));
            }
        });

        // Create CowArrays from vectors
        let inputs_ids_array = Array::from_shape_vec((batch_size, encoding_length), ids_array)
            .map_err(|e| AIProxyError::ModelProviderPreprocessingError(e.to_string()))?;

        let attention_mask_array = Array::from_shape_vec((batch_size, encoding_length), mask_array)
            .map_err(|e| AIProxyError::ModelProviderPreprocessingError(e.to_string()))?;

        let token_type_ids_array = match token_type_ids_array {
            Some(token_type_ids_array) => Some(
                Array::from_shape_vec((batch_size, encoding_length), token_type_ids_array)
                    .map_err(|e| AIProxyError::ModelProviderPreprocessingError(e.to_string()))?,
            ),
            None => None,
        };

        let mut session_inputs = ort::inputs![
            "input_ids" => Value::from_array(inputs_ids_array)?,
            "attention_mask" => Value::from_array(attention_mask_array.view())?
        ]
        .map_err(|e| AIProxyError::ModelProviderPreprocessingError(e.to_string()))?;

        if let Some(token_type_ids_array) = token_type_ids_array {
            session_inputs.push((
                "token_type_ids".into(),
                Value::from_array(token_type_ids_array)?.into(),
            ));
        }

        let child_span = tracing::info_span!("text-model-session-run");
        child_span.set_parent(Span::current().context());
        let child_guard = child_span.enter();
        let session_outputs = session
            .run(session_inputs)
            .map_err(|e| AIProxyError::ModelProviderRunInferenceError(e.to_string()))?;
        drop(child_guard);

        // Postprocess text output directly here
        let embeddings = self.postprocess_text_output(session_outputs, attention_mask_array)?;
        Ok(embeddings)
    }

    /// Postprocess text model output with mean pooling
    fn postprocess_text_output(
        &self,
        session_output: ort::SessionOutputs,
        attention_mask: Array<i64, Ix2>,
    ) -> Result<Array<f32, Ix2>, AIProxyError> {
        let output_tensor = session_output
            .values()
            .next()
            .ok_or_else(|| {
                AIProxyError::ModelProviderPostprocessingError("No output tensor found".to_string())
            })?
            .try_extract_tensor::<f32>()
            .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;

        let shape = output_tensor.shape();
        let batch_size = shape[0];
        let seq_len = shape[1];
        let hidden_size = shape[2];

        // Perform mean pooling
        let mut pooled = Array::zeros((batch_size, hidden_size));

        for b in 0..batch_size {
            let mut sum: Array<f32, _> = Array::zeros(hidden_size);
            let mut count = 0.0f32;

            for s in 0..seq_len {
                let mask_val = attention_mask[[b, s]] as f32;
                if mask_val > 0.0 {
                    for h in 0..hidden_size {
                        sum[h] += output_tensor[[b, s, h]] * mask_val;
                    }
                    count += mask_val;
                }
            }

            if count > 0.0 {
                for h in 0..hidden_size {
                    pooled[[b, h]] = sum[h] / count;
                }
            }
        }

        // Apply normalization if needed
        if matches!(
            self.supported_models,
            SupportedModels::AllMiniLML6V2
                | SupportedModels::AllMiniLML12V2
                | SupportedModels::BGEBaseEnV15
                | SupportedModels::BGELargeEnV15
        ) {
            self.normalize_embeddings(&mut pooled);
        }

        Ok(pooled)
    }
}

impl ORTInferenceModel for SingleStageModel {
    fn infer_batch(
        &self,
        input: ModelInput,
        execution_provider: Option<AIExecutionProvider>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ModelResponse>, AIProxyError>> + Send + '_>> {
        Box::pin(async move { self.infer_batch_impl(input, execution_provider).await })
    }

    fn batch_size(&self) -> usize {
        self.model_batch_size
    }
}

impl SingleStageModel {
    async fn infer_batch_impl(
        &self,
        input: ModelInput,
        execution_provider: Option<AIExecutionProvider>,
    ) -> Result<Vec<ModelResponse>, AIProxyError> {
        let session = if let Some(exec_prov) = execution_provider {
            self.executor_session_cache
                .try_get_with(exec_prov.into())
                .await?
        } else {
            self.executor_session_cache
                .try_get_with(super::InnerAIExecutionProvider::CPU)
                .await?
        };

        match (self.model_type, &input) {
            (ORTModality::Audio, ModelInput::Audios(_))
            | (ORTModality::Image, ModelInput::Images(_))
            | (ORTModality::Text, ModelInput::Texts(_)) => (),
            _ => {
                return Err(AIProxyError::AIModelNotSupported {
                    model_name: format!("{:?} model", self.model_type),
                });
            }
        };

        match input {
            ModelInput::Images(images) => {
                let mut store_keys: Vec<ModelResponse> =
                    FallibleVec::try_with_capacity(images.len())?;

                for batch_image in images.axis_chunks_iter(Axis(0), self.model_batch_size) {
                    let embeddings =
                        self.batch_inference_image(batch_image.to_owned(), &session)?;

                    // Memory check: Converting embeddings to Vec<ModelResponse>
                    // + 64: Vec overhead for StoreKey
                    let batch_size = embeddings.shape()[0];
                    let embedding_dim = embeddings.shape()[1];
                    let bytes_per_response = size_of::<ModelResponse>()
                        + size_of::<StoreKey>()
                        + (embedding_dim * size_of::<f32>())
                        + 64;
                    utils::allocator::check_memory_available(batch_size * bytes_per_response)
                        .map_err(|e| AIProxyError::Allocation(e.into()))?;

                    let new_store_keys: Vec<ModelResponse> = embeddings
                        .axis_iter(Axis(0))
                        .into_par_iter()
                        .map(|embedding: ArrayView1<f32>| -> ModelResponse {
                            ModelResponse::OneToOne(StoreKey {
                                key: embedding.to_vec(),
                            })
                        })
                        .collect();
                    store_keys.extend(new_store_keys);
                }
                Ok(store_keys)
            }
            ModelInput::Texts(encodings) => {
                let mut store_keys: Vec<ModelResponse> =
                    FallibleVec::try_with_capacity(encodings.len())?;

                for batch_encoding in encodings
                    .into_iter()
                    .chunks(self.model_batch_size)
                    .into_iter()
                {
                    let embeddings =
                        self.batch_inference_text(batch_encoding.collect(), &session)?;

                    // Memory check: Converting embeddings to Vec<ModelResponse>
                    // + 64: Vec overhead for StoreKey
                    let batch_size = embeddings.shape()[0];
                    let embedding_dim = embeddings.shape()[1];
                    let bytes_per_response = size_of::<ModelResponse>()
                        + size_of::<StoreKey>()
                        + (embedding_dim * size_of::<f32>())
                        + 64;
                    utils::allocator::check_memory_available(batch_size * bytes_per_response)
                        .map_err(|e| AIProxyError::Allocation(e.into()))?;

                    let new_store_keys: Vec<ModelResponse> = embeddings
                        .axis_iter(Axis(0))
                        .into_par_iter()
                        .map(|embedding: ArrayView1<f32>| -> ModelResponse {
                            ModelResponse::OneToOne(StoreKey {
                                key: embedding.to_vec(),
                            })
                        })
                        .collect();
                    store_keys.extend(new_store_keys);
                }
                Ok(store_keys)
            }
            ModelInput::Audios(waveforms) => {
                let mut store_keys: Vec<ModelResponse> =
                    FallibleVec::try_with_capacity(waveforms.shape()[0])?;

                for batch in waveforms.axis_chunks_iter(Axis(0), self.model_batch_size) {
                    let embeddings = self.batch_inference_audio(batch.to_owned(), &session)?;

                    let batch_size = embeddings.shape()[0];
                    let embedding_dim = embeddings.shape()[1];
                    let bytes_per_response = size_of::<ModelResponse>()
                        + size_of::<StoreKey>()
                        + (embedding_dim * size_of::<f32>())
                        + 64;
                    utils::allocator::check_memory_available(batch_size * bytes_per_response)
                        .map_err(|e| AIProxyError::Allocation(e.into()))?;

                    let new_store_keys: Vec<ModelResponse> = embeddings
                        .axis_iter(Axis(0))
                        .into_par_iter()
                        .map(|embedding: ArrayView1<f32>| -> ModelResponse {
                            ModelResponse::OneToOne(StoreKey {
                                key: embedding.to_vec(),
                            })
                        })
                        .collect();
                    store_keys.extend(new_store_keys);
                }
                Ok(store_keys)
            }
        }
    }

    #[tracing::instrument(skip(self, waveforms, session))]
    fn batch_inference_audio(
        &self,
        waveforms: ndarray::Array<f32, ndarray::Ix2>,
        session: &Session,
    ) -> Result<Array<f32, Ix2>, AIProxyError> {
        // CLAP audio encoder expects input named "input_features" with shape (B, samples)
        let session_inputs = ort::inputs![
            "input_features" => waveforms.view(),
        ]
        .map_err(|e| AIProxyError::ModelProviderPreprocessingError(e.to_string()))?;

        let child_span = tracing::info_span!("audio-model-session-run");
        child_span.set_parent(tracing::Span::current().context());
        let child_guard = child_span.enter();
        let outputs = session
            .run(session_inputs)
            .map_err(|e| AIProxyError::ModelProviderRunInferenceError(e.to_string()))?;
        drop(child_guard);

        let output_tensor = outputs
            .values()
            .next()
            .ok_or_else(|| {
                AIProxyError::ModelProviderPostprocessingError("No output tensor found".to_string())
            })?
            .try_extract_tensor::<f32>()
            .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;

        let mut embeddings = output_tensor
            .to_owned()
            .into_dimensionality::<Ix2>()
            .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;

        // L2-normalise so audio and text embeddings are comparable
        self.normalize_embeddings(&mut embeddings);

        Ok(embeddings)
    }
}
