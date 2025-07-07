mod executor;
pub(crate) mod helper;

use crate::cli::server::SupportedModels;
use crate::engine::ai::models::{ImageArray, InputAction, ModelInput};
use crate::engine::ai::providers::ProviderTrait;
use crate::error::AIProxyError;
use ahnlich_types::ai::execution_provider::ExecutionProvider as AIExecutionProvider;
use ahnlich_types::keyval::StoreKey;
use executor::ExecutorWithSessionCache;
use fallible_collections::FallibleVec;
use hf_hub::{api::sync::ApiBuilder, Cache};
use itertools::Itertools;
use ort::{
    CUDAExecutionProvider, CoreMLExecutionProvider, DirectMLExecutionProvider, ExecutionProvider,
    SessionBuilder, TensorRTExecutionProvider,
};
use ort::{Session, SessionOutputs, Value};
use rayon::prelude::*;
use strum::EnumIter;
use tracing::Span;
use tracing_opentelemetry::OpenTelemetrySpanExt;

use crate::engine::ai::providers::processors::postprocessor::{
    ORTImagePostprocessor, ORTPostprocessor, ORTTextPostprocessor,
};
use crate::engine::ai::providers::processors::preprocessor::{
    ORTImagePreprocessor, ORTPreprocessor, ORTTextPreprocessor,
};
use ndarray::{Array, Axis, Ix2, Ix4};
use std::convert::TryFrom;
use std::default::Default;
use std::path::{Path, PathBuf};
use tokenizers::Encoding;

pub struct ORTProvider {
    cache_location: PathBuf,
    supported_models: SupportedModels,
    pub preprocessor: ORTPreprocessor,
    pub postprocessor: ORTPostprocessor,
    pub model: ORTModel,
}

#[derive(EnumIter, Default, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Hash, Ord)]
#[allow(clippy::upper_case_acronyms)]
pub(crate) enum InnerAIExecutionProvider {
    TensorRT,
    CUDA,
    DirectML,
    CoreML,
    #[default]
    CPU,
}

impl From<AIExecutionProvider> for InnerAIExecutionProvider {
    fn from(entry: AIExecutionProvider) -> Self {
        match entry {
            AIExecutionProvider::TensorRt => InnerAIExecutionProvider::TensorRT,
            AIExecutionProvider::Cuda => InnerAIExecutionProvider::CUDA,
            AIExecutionProvider::DirectMl => InnerAIExecutionProvider::DirectML,
            AIExecutionProvider::CoreMl => InnerAIExecutionProvider::CoreML,
        }
    }
}

fn register_provider(
    provider: InnerAIExecutionProvider,
    builder: &SessionBuilder,
) -> Result<(), AIProxyError> {
    match provider {
        InnerAIExecutionProvider::TensorRT => {
            TensorRTExecutionProvider::default().register(builder)?
        }
        InnerAIExecutionProvider::CUDA => CUDAExecutionProvider::default().register(builder)?,
        InnerAIExecutionProvider::DirectML => {
            DirectMLExecutionProvider::default().register(builder)?
        }
        InnerAIExecutionProvider::CoreML => CoreMLExecutionProvider::default().register(builder)?,
        InnerAIExecutionProvider::CPU => (),
    };
    Ok(())
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Hash, Ord)]
enum ORTModality {
    Image,
    Text,
}

pub struct ORTModel {
    model_type: ORTModality,
    repo_name: String,
    weights_file: String,
    executor_session_cache: Option<ExecutorWithSessionCache>,
    model_batch_size: usize,
}

impl Default for ORTModel {
    fn default() -> Self {
        Self {
            model_type: ORTModality::Text,
            repo_name: "".to_string(),
            weights_file: "model.onnx".to_string(),
            executor_session_cache: None,
            model_batch_size: 128,
        }
    }
}

impl TryFrom<&SupportedModels> for ORTModel {
    type Error = AIProxyError;

    fn try_from(model: &SupportedModels) -> Result<Self, Self::Error> {
        let model_type: Result<ORTModel, AIProxyError> = match model {
            SupportedModels::Resnet50 => Ok(ORTModel {
                model_type: ORTModality::Image,
                repo_name: "Qdrant/resnet50-onnx".to_string(),
                model_batch_size: 16,
                ..Default::default()
            }),
            SupportedModels::ClipVitB32Image => Ok(ORTModel {
                model_type: ORTModality::Image,
                repo_name: "Qdrant/clip-ViT-B-32-vision".to_string(),
                model_batch_size: 16,
                ..Default::default()
            }),
            SupportedModels::ClipVitB32Text => Ok(ORTModel {
                model_type: ORTModality::Text,
                repo_name: "Qdrant/clip-ViT-B-32-text".to_string(),
                ..Default::default()
            }),
            SupportedModels::AllMiniLML6V2 => Ok(ORTModel {
                model_type: ORTModality::Text,
                repo_name: "Qdrant/all-MiniLM-L6-v2-onnx".to_string(),
                ..Default::default()
            }),
            SupportedModels::AllMiniLML12V2 => Ok(ORTModel {
                model_type: ORTModality::Text,
                repo_name: "Xenova/all-MiniLM-L12-v2".to_string(),
                weights_file: "onnx/model.onnx".to_string(),
                ..Default::default()
            }),
            SupportedModels::BGEBaseEnV15 => Ok(ORTModel {
                model_type: ORTModality::Text,
                repo_name: "Xenova/bge-base-en-v1.5".to_string(),
                weights_file: "onnx/model.onnx".to_string(),
                ..Default::default()
            }),
            SupportedModels::BGELargeEnV15 => Ok(ORTModel {
                model_type: ORTModality::Text,
                repo_name: "Xenova/bge-large-en-v1.5".to_string(),
                weights_file: "onnx/model.onnx".to_string(),
                ..Default::default()
            }),
        };

        model_type
    }
}

fn ort_full_cache_path(cache_location: &Path) -> PathBuf {
    cache_location.join(PathBuf::from("huggingface"))
}

impl ORTProvider {
    fn cache_path(&self) -> PathBuf {
        ort_full_cache_path(&self.cache_location)
    }

    #[tracing::instrument]
    pub(crate) async fn from_model_and_cache_location(
        supported_models: &SupportedModels,
        cache_location: PathBuf,
        session_profiling: bool,
    ) -> Result<Self, AIProxyError> {
        let mut model = ORTModel::try_from(supported_models)?;
        let cache = Cache::new(ort_full_cache_path(&cache_location));
        let api = ApiBuilder::from_cache(cache)
            .with_progress(true)
            .build()
            .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?;

        let model_repo = api.model(model.repo_name.clone());
        let model_file_reference = model_repo
            .get(&model.weights_file)
            .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?;
        // Warm the cache with a default CPU provider
        let executor_with_session_cache =
            ExecutorWithSessionCache::new(model_file_reference, session_profiling);
        executor_with_session_cache
            .try_get_with(InnerAIExecutionProvider::CPU)
            .await?;
        model.executor_session_cache = Some(executor_with_session_cache);
        let (preprocessor, postprocessor) = match model.model_type {
            ORTModality::Image => {
                let preprocessor = ORTPreprocessor::Image(ORTImagePreprocessor::load(
                    *supported_models,
                    model_repo,
                )?);
                let postprocessor =
                    ORTPostprocessor::Image(ORTImagePostprocessor::load(*supported_models)?);
                (preprocessor, postprocessor)
            }
            ORTModality::Text => {
                let preprocessor = ORTPreprocessor::Text(ORTTextPreprocessor::load(
                    *supported_models,
                    model_repo,
                )?);
                let postprocessor =
                    ORTPostprocessor::Text(ORTTextPostprocessor::load(*supported_models)?);
                (preprocessor, postprocessor)
            }
        };
        Ok(Self {
            cache_location,
            supported_models: *supported_models,
            postprocessor,
            preprocessor,
            model,
        })
    }

    #[tracing::instrument(skip_all)]
    pub fn preprocess_images(
        &self,
        data: Vec<ImageArray>,
    ) -> Result<Array<f32, Ix4>, AIProxyError> {
        match &self.preprocessor {
            ORTPreprocessor::Image(preprocessor) => {
                let output_data = preprocessor.process(data).map_err(|e| {
                    AIProxyError::ModelProviderPreprocessingError(format!(
                        "Preprocessing failed for {:?} with error: {}",
                        self.supported_models.to_string(),
                        e
                    ))
                })?;
                Ok(output_data)
            }
            _ => Err(AIProxyError::AIModelNotInitialized),
        }
    }

    #[tracing::instrument(skip(self, data))]
    pub fn preprocess_texts(
        &self,
        data: Vec<String>,
        truncate: bool,
    ) -> Result<Vec<Encoding>, AIProxyError> {
        match &self.preprocessor {
            ORTPreprocessor::Text(preprocessor) => {
                let output_data = preprocessor.process(data, truncate).map_err(|e| {
                    AIProxyError::ModelProviderPreprocessingError(format!(
                        "Preprocessing failed for {:?} with error: {}",
                        self.supported_models.to_string(),
                        e
                    ))
                })?;
                Ok(output_data)
            }
            _ => Err(AIProxyError::ModelPreprocessingError {
                model_name: self.supported_models.to_string(),
                message: "Preprocessor not initialized".to_string(),
            }),
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn postprocess_text_output(
        &self,
        session_output: SessionOutputs,
        attention_mask: Array<i64, Ix2>,
    ) -> Result<Array<f32, Ix2>, AIProxyError> {
        match &self.postprocessor {
            ORTPostprocessor::Text(postprocessor) => {
                let output_data = postprocessor
                    .process(session_output, attention_mask)
                    .map_err(|e| {
                        AIProxyError::ModelProviderPostprocessingError(format!(
                            "Postprocessing failed for {:?} with error: {}",
                            self.supported_models.to_string(),
                            e
                        ))
                    })?;
                Ok(output_data)
            }
            _ => Err(AIProxyError::ModelPostprocessingError {
                model_name: self.supported_models.to_string(),
                message: "Postprocessor not initialized".to_string(),
            }),
        }
    }

    #[tracing::instrument(skip_all)]
    pub fn postprocess_image_output(
        &self,
        session_output: SessionOutputs,
    ) -> Result<Array<f32, Ix2>, AIProxyError> {
        match &self.postprocessor {
            ORTPostprocessor::Image(postprocessor) => {
                let output_data = postprocessor.process(session_output).map_err(|e| {
                    AIProxyError::ModelProviderPostprocessingError(format!(
                        "Postprocessing failed for {:?} with error: {}",
                        self.supported_models.to_string(),
                        e
                    ))
                })?;
                Ok(output_data)
            }
            _ => Err(AIProxyError::ModelPostprocessingError {
                model_name: self.supported_models.to_string(),
                message: "Postprocessor not initialized".to_string(),
            }),
        }
    }

    #[tracing::instrument(skip(self, inputs, session))]
    pub fn batch_inference_image(
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
                })
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
        let embeddings = self.postprocess_image_output(outputs)?;
        Ok(embeddings)
    }

    #[tracing::instrument(skip(self, encodings))]
    pub fn batch_inference_text(
        &self,
        encodings: Vec<Encoding>,
        session: &Session,
    ) -> Result<Array<f32, Ix2>, AIProxyError> {
        if self.model.model_type != ORTModality::Text {
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
        // Preallocate arrays with the maximum size
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
        let embeddings = self.postprocess_text_output(session_outputs, attention_mask_array)?;
        Ok(embeddings.to_owned())
    }
}

#[async_trait::async_trait]
impl ProviderTrait for ORTProvider {
    async fn get_model(&self) -> Result<(), AIProxyError> {
        let cache = Cache::new(self.cache_path());
        let api = ApiBuilder::from_cache(cache)
            .with_progress(true)
            .build()
            .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?;

        let model_repo = api.model(self.model.repo_name.clone());
        model_repo
            .get(&self.model.weights_file)
            .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?;
        Ok(())
    }

    #[tracing::instrument(skip(self, input), fields(model_batch_size = self.model.model_batch_size))]
    async fn run_inference(
        &self,
        input: ModelInput,
        _action_type: &InputAction,
        execution_provider: Option<AIExecutionProvider>,
    ) -> Result<Vec<StoreKey>, AIProxyError> {
        let session = match &self.model.executor_session_cache {
            Some(executor_session_cache) => {
                if let Some(execution_provider) = execution_provider {
                    executor_session_cache
                        .try_get_with(execution_provider.into())
                        .await?
                } else {
                    executor_session_cache
                        .try_get_with(InnerAIExecutionProvider::CPU)
                        .await?
                }
            }
            None => return Err(AIProxyError::AIModelNotInitialized),
        };
        match (self.model.model_type, &input) {
            (ORTModality::Text, ModelInput::Images(_))
            | (ORTModality::Image, ModelInput::Texts(_)) => {
                return Err(AIProxyError::AIModelNotSupported {
                    model_name: self.supported_models.to_string(),
                });
            }
            (ORTModality::Image, ModelInput::Images(_))
            | (ORTModality::Text, ModelInput::Texts(_)) => (),
        };
        match input {
            ModelInput::Images(images) => {
                let mut store_keys: Vec<StoreKey> = FallibleVec::try_with_capacity(images.len())?;

                for batch_image in images.axis_chunks_iter(Axis(0), self.model.model_batch_size) {
                    let embeddings =
                        self.batch_inference_image(batch_image.to_owned(), &session)?;
                    let new_store_keys: Vec<StoreKey> = embeddings
                        .axis_iter(Axis(0))
                        .into_par_iter()
                        .map(|embedding| StoreKey {
                            key: embedding.to_vec(),
                        })
                        .collect();
                    store_keys.extend(new_store_keys);
                }
                Ok(store_keys)
            }
            ModelInput::Texts(encodings) => {
                let mut store_keys: Vec<StoreKey> =
                    FallibleVec::try_with_capacity(encodings.len())?;

                for batch_encoding in encodings
                    .into_iter()
                    .chunks(self.model.model_batch_size)
                    .into_iter()
                {
                    let embeddings =
                        self.batch_inference_text(batch_encoding.collect(), &session)?;
                    let new_store_keys: Vec<StoreKey> = embeddings
                        .axis_iter(Axis(0))
                        .into_par_iter()
                        .map(|embedding| StoreKey {
                            key: embedding.to_vec(),
                        })
                        .collect();
                    store_keys.extend(new_store_keys);
                }
                Ok(store_keys)
            }
        }
    }
}
