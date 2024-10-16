use crate::cli::server::SupportedModels;
use crate::engine::ai::models::{InputAction, Model, ModelInput};
use crate::engine::ai::providers::{ProviderTrait, TextPreprocessorTrait};
use crate::error::AIProxyError;
use ahnlich_types::ai::AIStoreInputType;
use fastembed::{EmbeddingModel, ImageEmbedding, InitOptions, TextEmbedding};
use hf_hub::{api::sync::ApiBuilder, Cache};
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
    fn new(supported_models: &SupportedModels) -> Self {
        let _ = supported_models;
        // Using ChatGPT model tokenizers as a default till we add specific implementations.
        let bpe = cl100k_base().unwrap();
        Tokenizer(bpe)
    }
}

pub struct FastEmbedPreprocessor {
    tokenizer: Tokenizer,
}

// TODO (HAKSOAT): Implement other preprocessors
impl TextPreprocessorTrait for FastEmbedPreprocessor {
    fn encode_str(&self, text: &str) -> Result<Vec<usize>, AIProxyError> {
        let tokens = self.tokenizer.0.encode_with_special_tokens(text);
        Ok(tokens)
    }

    fn decode_tokens(&self, tokens: Vec<usize>) -> Result<String, AIProxyError> {
        let text = self
            .tokenizer
            .0
            .decode(tokens)
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
        let model_type = match model_modality {
            Model::Text { .. } => {
                let model_type = match model {
                    SupportedModels::AllMiniLML6V2 => EmbeddingModel::AllMiniLML6V2,
                    SupportedModels::AllMiniLML12V2 => EmbeddingModel::AllMiniLML12V2,
                    SupportedModels::BGEBaseEnV15 => EmbeddingModel::BGEBaseENV15,
                    SupportedModels::BGELargeEnV15 => EmbeddingModel::BGELargeENV15,
                    _ => return Err(AIProxyError::AIModelNotSupported),
                };
                FastEmbedModelType::Text(model_type)
            }
            _ => return Err(AIProxyError::AIModelNotSupported),
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

    fn load_tokenizer(&mut self) -> &mut Self {
        let supported_models = self.supported_models.expect("Model has not been set.");
        let tokenizer = Tokenizer::new(&supported_models);
        self.preprocessor = Some(FastEmbedPreprocessor { tokenizer });
        self
    }

    pub fn encode_str(&self, text: &str) -> Result<Vec<usize>, AIProxyError> {
        if let Some(preprocessor) = &self.preprocessor {
            preprocessor.encode_str(text)
        } else {
            Err(AIProxyError::AIModelNotInitialized)
        }
    }

    pub fn decode_tokens(&self, tokens: Vec<usize>) -> Result<String, AIProxyError> {
        if let Some(preprocessor) = &self.preprocessor {
            preprocessor.decode_tokens(tokens)
        } else {
            Err(AIProxyError::AIModelNotInitialized)
        }
    }
}

impl ProviderTrait for FastEmbedProvider {
    fn set_cache_location(&mut self, location: &Path) -> &mut Self {
        self.cache_location = Some(location.join(self.cache_location_extension.clone()));
        self
    }

    fn set_model(&mut self, model: &SupportedModels) -> &mut Self {
        self.supported_models = Some(*model);
        self
    }

    fn load_model(&mut self) -> &mut Self {
        let cache_location = self
            .cache_location
            .clone()
            .expect("Cache location not set.");
        let model_type = self.supported_models.expect("Model has not been set.");
        let model_type = FastEmbedModelType::try_from(&model_type).unwrap_or_else(|_| {
            panic!(
                "This provider does not support the model: {:?}.",
                model_type
            )
        });

        let FastEmbedModelType::Text(model_type) = model_type;
        let model =
            TextEmbedding::try_new(InitOptions::new(model_type).with_cache_dir(cache_location))
                .unwrap();
        self.model = Some(FastEmbedModel::Text(model));
        self.load_tokenizer();
        self
    }

    fn get_model(&self) {
        let cache_location = self
            .cache_location
            .clone()
            .expect("Cache location not set.");
        let model_type = self
            .supported_models
            .expect(&AIProxyError::AIModelNotInitialized.to_string());
        let model_type = FastEmbedModelType::try_from(&model_type).unwrap_or_else(|_| {
            panic!(
                "This provider does not support the model: {:?}.",
                model_type
            )
        });

        match model_type {
            FastEmbedModelType::Text(model_type) => {
                let cache = Cache::new(cache_location);
                let api = ApiBuilder::from_cache(cache)
                    .with_progress(true)
                    .build()
                    .unwrap();
                let model_repo = api.model(model_type.to_string());
                let model_info = TextEmbedding::get_model_info(&model_type).unwrap();
                model_repo.get(model_info.model_file.as_str()).unwrap();
            }
        }
        //     TODO (HAKSOAT): When we add model specific tokenizers, add the get tokenizer call here too.
    }

    fn run_inference(
        &self,
        input: &ModelInput,
        action_type: &InputAction,
    ) -> Result<Vec<f32>, AIProxyError> {
        if let Some(fastembed_model) = &self.model {
            let embeddings = match fastembed_model {
                FastEmbedModel::Text(model) => {
                    if let ModelInput::Text(value) = input {
                        model
                            .embed(vec![value], None)
                            .map_err(|_| AIProxyError::ModelProviderRunInferenceError)?
                    } else {
                        let store_input_type: AIStoreInputType = input.into();
                        let index_model_repr: Model = (&self
                            .supported_models
                            .expect(&AIProxyError::AIModelNotInitialized.to_string()))
                            .into();
                        return Err(AIProxyError::StoreTypeMismatchError {
                            action: *action_type,
                            index_model_type: index_model_repr.input_type(),
                            storeinput_type: store_input_type,
                        });
                    }
                }
                _ => return Err(AIProxyError::AIModelNotSupported),
            };
            let embeddings = embeddings
                .first()
                .ok_or(AIProxyError::ModelProviderPostprocessingError)?;
            return Ok(embeddings.to_owned());
        } else {
            return Err(AIProxyError::AIModelNotSupported);
        }
    }
}
