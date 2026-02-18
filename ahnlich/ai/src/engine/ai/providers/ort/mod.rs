mod executor;
pub(crate) mod helper;
mod inference_model;
mod models;
mod single_stage;

use inference_model::{ORTInferenceModel, ORTModality};
use models::buffalo_l::BuffaloLModel;
use single_stage::SingleStageModel;

use crate::cli::server::SupportedModels;
use crate::engine::ai::models::{ImageArray, InputAction, ModelInput, ModelResponse};
use crate::engine::ai::providers::ProviderTrait;
use crate::error::AIProxyError;
use ahnlich_types::ai::execution_provider::ExecutionProvider as AIExecutionProvider;
use executor::ExecutorWithSessionCache;
use hf_hub::{Cache, api::sync::ApiBuilder};
use ort::{
    CUDAExecutionProvider, CoreMLExecutionProvider, DirectMLExecutionProvider, ExecutionProvider,
    SessionBuilder, SessionOutputs, TensorRTExecutionProvider,
};
use strum::EnumIter;

use crate::engine::ai::providers::processors::AudioInput;
use crate::engine::ai::providers::processors::postprocessor::{
    ORTImagePostprocessor, ORTPostprocessor, ORTTextPostprocessor,
};
use crate::engine::ai::providers::processors::preprocessor::{
    ORTAudioPreprocessor, ORTImagePreprocessor, ORTPreprocessor, ORTTextPreprocessor,
};
use ndarray::{Array, Ix2, Ix4};
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

pub struct ORTModel {
    inner: Box<dyn ORTInferenceModel>,
}

impl ORTModel {
    /// Returns the batch size for this model
    pub fn batch_size(&self) -> usize {
        self.inner.batch_size()
    }
}

// Model configuration helper
struct ModelConfig {
    modality: ORTModality,
    repo_name: String,
    weights_file: String,
    batch_size: usize,
}

impl ModelConfig {
    fn from_supported_model(model: &SupportedModels) -> Result<Self, AIProxyError> {
        Ok(match model {
            SupportedModels::Resnet50 => Self {
                modality: ORTModality::Image,
                repo_name: "Qdrant/resnet50-onnx".to_string(),
                weights_file: "model.onnx".to_string(),
                batch_size: 16,
            },
            SupportedModels::ClipVitB32Image => Self {
                modality: ORTModality::Image,
                repo_name: "Qdrant/clip-ViT-B-32-vision".to_string(),
                weights_file: "model.onnx".to_string(),
                batch_size: 16,
            },
            SupportedModels::ClipVitB32Text => Self {
                modality: ORTModality::Text,
                repo_name: "Qdrant/clip-ViT-B-32-text".to_string(),
                weights_file: "model.onnx".to_string(),
                batch_size: 128,
            },
            SupportedModels::AllMiniLML6V2 => Self {
                modality: ORTModality::Text,
                repo_name: "Qdrant/all-MiniLM-L6-v2-onnx".to_string(),
                weights_file: "model.onnx".to_string(),
                batch_size: 128,
            },
            SupportedModels::AllMiniLML12V2 => Self {
                modality: ORTModality::Text,
                repo_name: "Xenova/all-MiniLM-L12-v2".to_string(),
                weights_file: "onnx/model.onnx".to_string(),
                batch_size: 128,
            },
            SupportedModels::BGEBaseEnV15 => Self {
                modality: ORTModality::Text,
                repo_name: "Xenova/bge-base-en-v1.5".to_string(),
                weights_file: "onnx/model.onnx".to_string(),
                batch_size: 128,
            },
            SupportedModels::BGELargeEnV15 => Self {
                modality: ORTModality::Text,
                repo_name: "Xenova/bge-large-en-v1.5".to_string(),
                weights_file: "onnx/model.onnx".to_string(),
                batch_size: 128,
            },
            SupportedModels::ClapAudio => Self {
                modality: ORTModality::Audio,
                repo_name: "Xenova/larger_clap_music_and_speech".to_string(),
                weights_file: "onnx/audio_model.onnx".to_string(),
                batch_size: 8,
            },
            SupportedModels::ClapText => Self {
                modality: ORTModality::Text,
                repo_name: "Xenova/larger_clap_music_and_speech".to_string(),
                weights_file: "onnx/text_model.onnx".to_string(),
                batch_size: 128,
            },
            SupportedModels::BuffaloL => {
                // BuffaloL is a multi-stage model and doesn't use ModelConfig
                // This branch should never be reached as Buffalo_L is handled separately
                return Err(AIProxyError::AIModelNotInitialized);
            }
        })
    }
}

fn ort_full_cache_path(cache_location: &Path) -> PathBuf {
    cache_location.join(PathBuf::from("huggingface"))
}

impl ORTProvider {
    #[allow(dead_code)]
    fn cache_path(&self) -> PathBuf {
        ort_full_cache_path(&self.cache_location)
    }

    #[tracing::instrument]
    pub(crate) async fn from_model_and_cache_location(
        supported_models: &SupportedModels,
        cache_location: PathBuf,
        session_profiling: bool,
    ) -> Result<Self, AIProxyError> {
        let cache = Cache::new(ort_full_cache_path(&cache_location));
        let api = ApiBuilder::from_cache(cache)
            .with_progress(true)
            .build()
            .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?;

        // Handle multi-stage models separately
        match supported_models {
            SupportedModels::BuffaloL => {
                // Build BuffaloLModel (multi-stage)
                let inner = BuffaloLModel::build(api.clone(), session_profiling).await?;
                let model = ORTModel { inner };

                let model_repo = api.model("immich-app/buffalo_l".to_string());
                let preprocessor = ORTPreprocessor::Image(ORTImagePreprocessor::load(
                    *supported_models,
                    model_repo,
                )?);
                let postprocessor =
                    ORTPostprocessor::Image(ORTImagePostprocessor::load(*supported_models)?);

                Ok(Self {
                    cache_location,
                    supported_models: *supported_models,
                    postprocessor,
                    preprocessor,
                    model,
                })
            }
            // All other models are single-stage
            _ => {
                let config = ModelConfig::from_supported_model(supported_models)?;
                let model_repo = api.model(config.repo_name.clone());
                let model_file_reference = model_repo
                    .get(&config.weights_file)
                    .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?;

                // Build SingleStageModel
                let executor_with_session_cache =
                    ExecutorWithSessionCache::new(model_file_reference, session_profiling);
                executor_with_session_cache
                    .try_get_with(InnerAIExecutionProvider::CPU)
                    .await?;

                let inner: Box<dyn ORTInferenceModel> = Box::new(SingleStageModel::new(
                    config.modality,
                    executor_with_session_cache,
                    config.batch_size,
                    *supported_models,
                ));

                let model = ORTModel { inner };

                let (preprocessor, postprocessor) = match config.modality {
                    ORTModality::Image => {
                        let preprocessor = ORTPreprocessor::Image(ORTImagePreprocessor::load(
                            *supported_models,
                            model_repo,
                        )?);
                        let postprocessor = ORTPostprocessor::Image(ORTImagePostprocessor::load(
                            *supported_models,
                        )?);
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
                    ORTModality::Audio => {
                        // CLAP: 48kHz, 10-second clips
                        let preprocessor = ORTPreprocessor::Audio(ORTAudioPreprocessor::new(
                            *supported_models,
                            48_000,
                            10.0,
                        ));
                        // Audio postprocessing is handled inline in batch_inference_audio
                        let postprocessor = ORTPostprocessor::Audio;
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
        }
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

    #[tracing::instrument(skip(self, data))]
    pub fn preprocess_audios(&self, data: Vec<Vec<u8>>) -> Result<AudioInput, AIProxyError> {
        match &self.preprocessor {
            ORTPreprocessor::Audio(preprocessor) => preprocessor.process(data).map_err(|e| {
                AIProxyError::ModelProviderPreprocessingError(format!(
                    "Audio preprocessing failed for {:?}: {}",
                    self.supported_models.to_string(),
                    e
                ))
            }),
            _ => Err(AIProxyError::ModelPreprocessingError {
                model_name: self.supported_models.to_string(),
                message: "Audio preprocessor not initialized".to_string(),
            }),
        }
    }
}

#[async_trait::async_trait]
impl ProviderTrait for ORTProvider {
    async fn get_model(&self) -> Result<(), AIProxyError> {
        // Model is already loaded in from_model_and_cache_location
        Ok(())
    }

    #[tracing::instrument(skip(self, input))]
    async fn run_inference(
        &self,
        input: ModelInput,
        _action_type: &InputAction,
        execution_provider: Option<AIExecutionProvider>,
    ) -> Result<Vec<ModelResponse>, AIProxyError> {
        self.model
            .inner
            .infer_batch(input, execution_provider)
            .await
    }
}
