use crate::cli::server::SupportedModels;
use crate::engine::ai::models::{ImageArray, InputAction, Model, ModelInput, ModelType};
use crate::engine::ai::providers::{ProviderTrait, TextPreprocessorTrait};
use crate::error::AIProxyError;
use ahnlich_types::ai::AIStoreInputType;
use ahnlich_types::keyval::StoreKey;
use fastembed::{EmbeddingModel, ImageEmbedding, InitOptions, TextEmbedding};
use hf_hub::{api::sync::ApiBuilder, Cache};
use ndarray::Array1;
use rayon::iter::Either;
use rayon::prelude::*;
use std::convert::TryFrom;
use std::fmt;
use std::path::{Path, PathBuf};
use tiktoken_rs::{cl100k_base, CoreBPE};

#[derive(Default)]
pub struct FastEmbedProvider {
    cache_location: Option<PathBuf>,
    cache_location_extension: PathBuf,
    supported_models: Option<SupportedModels>,
    pub preprocessor: Option<FastEmbedPreprocessor>,
    pub model: Option<FastEmbedModel>,
}

struct Tokenizer(CoreBPE);

// TODO (HAKSOAT): Implement Tokenizers specific to models
impl Tokenizer {
    fn new(supported_models: &SupportedModels) -> Result<Self, AIProxyError> {
        let _ = supported_models;
        // Using ChatGPT model tokenizers as a default till we add specific implementations.
        let bpe = cl100k_base().map_err(|e| AIProxyError::TokenizerInitError(e.to_string()))?;
        Ok(Tokenizer(bpe))
    }
}

pub struct FastEmbedPreprocessor {
    tokenizer: Tokenizer,
}

// TODO (HAKSOAT): Implement other preprocessors
impl TextPreprocessorTrait for FastEmbedProvider {
    fn encode_str(&self, text: &str) -> Result<Vec<usize>, AIProxyError> {
        let preprocessor = self.preprocessor.as_ref()
            .ok_or(AIProxyError::AIModelNotInitialized)?;
        let tokens = preprocessor
            .tokenizer.0.encode_with_special_tokens(text);
        Ok(tokens)
    }

    fn decode_tokens(&self, tokens: Vec<usize>) -> Result<String, AIProxyError> {
        let preprocessor = self.preprocessor.as_ref()
            .ok_or(AIProxyError::AIModelNotInitialized)?;
        let text = preprocessor
            .tokenizer.0.decode(tokens)
            .map_err(|_| AIProxyError::ModelTokenizationError)?;
        Ok(text)
    }
}

pub enum FastEmbedModelType {
    Text(EmbeddingModel),
}

impl fmt::Debug for FastEmbedProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("FastEmbedProvider")
            .field("cache_location", &self.cache_location)
            .field("cache_location_extension", &self.cache_location_extension)
            .field("supported_models", &self.supported_models)
            .finish()
    }
}

pub enum FastEmbedModel {
    Text(TextEmbedding),
    Image(ImageEmbedding),
}

impl TryFrom<&SupportedModels> for FastEmbedModelType {
    type Error = AIProxyError;

    fn try_from(model: &SupportedModels) -> Result<Self, Self::Error> {
        let model_modality: Model = model.into();
        let model_type = match model_modality.model_type {
            ModelType::Text { .. } => {
                let model_type = match model {
                    SupportedModels::AllMiniLML6V2 => EmbeddingModel::AllMiniLML6V2,
                    SupportedModels::AllMiniLML12V2 => EmbeddingModel::AllMiniLML12V2,
                    SupportedModels::BGEBaseEnV15 => EmbeddingModel::BGEBaseENV15,
                    SupportedModels::BGELargeEnV15 => EmbeddingModel::BGELargeENV15,
                    _ => return Err(AIProxyError::AIModelNotSupported { model_name: model.to_string() }),
                };
                FastEmbedModelType::Text(model_type)
            }
            _ => return Err(AIProxyError::AIModelNotSupported { model_name: model.to_string() }),
        };
        Ok(model_type)
    }
}

impl FastEmbedProvider {
    pub(crate) fn new() -> Self {
        FastEmbedProvider {
            cache_location: None,
            cache_location_extension: PathBuf::from("huggingface".to_string()),
            supported_models: None,
            model: None,
            preprocessor: None,
        }
    }

    fn load_tokenizer(&mut self) -> Result<(), AIProxyError> {
        let Some(supported_models) = self.supported_models else {
            return Err(AIProxyError::AIModelNotInitialized);
        };
        let tokenizer = Tokenizer::new(&supported_models)?;
        self.preprocessor = Some(FastEmbedPreprocessor { tokenizer });
        Ok(())
    }
}

impl ProviderTrait for FastEmbedProvider {
    fn set_cache_location(&mut self, location: &Path) {
        self.cache_location = Some(location.join(self.cache_location_extension.clone()));
    }

    fn set_model(&mut self, model: &SupportedModels) {
        self.supported_models = Some(*model);
    }

    fn load_model(&mut self) -> Result<(), AIProxyError> {
        let Some(cache_location) = self.cache_location.clone() else {
            return Err(AIProxyError::CacheLocationNotInitiailized);
        };
        let Some(model_type) = self.supported_models else {
            return Err(AIProxyError::AIModelNotInitialized);
        };
        let model_type = FastEmbedModelType::try_from(&model_type)?;
        let FastEmbedModelType::Text(model_type) = model_type;
        let model =
            TextEmbedding::try_new(InitOptions::new(model_type).with_cache_dir(cache_location))
                .map_err(|e| AIProxyError::TextEmbeddingInitError(e.to_string()))?;
        self.model = Some(FastEmbedModel::Text(model));
        self.load_tokenizer()
    }

    fn get_model(&self) -> Result<(), AIProxyError> {
        let Some(cache_location) = self.cache_location.clone() else {
            return Err(AIProxyError::CacheLocationNotInitiailized);
        };
        let Some(model_type) = self.supported_models else {
            return Err(AIProxyError::AIModelNotInitialized);
        };
        let model_type = FastEmbedModelType::try_from(&model_type)?;
        match model_type {
            FastEmbedModelType::Text(model_type) => {
                let cache = Cache::new(cache_location);
                let api = ApiBuilder::from_cache(cache)
                    .with_progress(true)
                    .build()
                    .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?;
                let model_repo = api.model(model_type.to_string());
                let model_info = TextEmbedding::get_model_info(&model_type)
                    .map_err(|e| AIProxyError::TextEmbeddingInitError(e.to_string()))?;
                model_repo
                    .get(model_info.model_file.as_str())
                    .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?;
                Ok(())
            }
        }
        //     TODO (HAKSOAT): When we add model specific tokenizers, add the get tokenizer call here too.
    }

    fn run_inference(
        &self,
        inputs: Vec<ModelInput>,
        action_type: &InputAction,
    ) -> Result<Vec<StoreKey>, AIProxyError> {
        return if let Some(fastembed_model) = &self.model {
            let (string_inputs, image_inputs): (Vec<String>, Vec<ImageArray>) =
                inputs.into_par_iter().partition_map(|input| match input {
                    ModelInput::Text(value) => Either::Left(value),
                    ModelInput::Image(value) => Either::Right(value),
                });

            if !image_inputs.is_empty() {
                let store_input_type: AIStoreInputType = AIStoreInputType::Image;
                let Some(index_model_repr) = self.supported_models else {
                    return Err(AIProxyError::AIModelNotInitialized);
                };
                let index_model_repr: Model = (&index_model_repr).into();
                return Err(AIProxyError::StoreTypeMismatchError {
                    action: *action_type,
                    index_model_type: index_model_repr.input_type(),
                    storeinput_type: store_input_type,
                });
            }
            let FastEmbedModel::Text(model) = fastembed_model else {
                return Err(AIProxyError::AIModelNotSupported { model_name: self.supported_models.unwrap().to_string() });
            };
            let batch_size = 16;
            let store_keys = model
                .embed(string_inputs, Some(batch_size))
                .map_err(|e| AIProxyError::ModelProviderRunInferenceError(e.to_string()))?
                .iter()
                .try_fold(Vec::new(), |mut accumulator, embedding| {
                    accumulator.push(StoreKey(<Array1<f32>>::from(embedding.to_owned())));
                    Ok(accumulator)
                });
            store_keys
        } else {
            Err(AIProxyError::AIModelNotSupported {
                model_name: self.supported_models.unwrap().to_string(),
            })
        };
    }
}
