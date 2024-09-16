use crate::cli::server::SupportedModels;
use crate::engine::ai::providers::fastembed::FastEmbedProvider;
use crate::engine::ai::providers::ModelProviders;
use crate::engine::ai::providers::ProviderTrait;
use crate::error::AIProxyError;
use ahnlich_types::{
    ai::{AIModel, AIStoreInputType},
    keyval::{StoreInput, StoreKey},
};
use ndarray::Array1;
use nonzero_ext::nonzero;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::num::NonZeroUsize;
use std::path::PathBuf;

pub enum Model {
    Text {
        supported_model: SupportedModels,
        description: String,
        embedding_size: NonZeroUsize,
        max_input_tokens: NonZeroUsize,
        provider: ModelProviders,
    },
    Image {
        supported_model: SupportedModels,
        description: String,
        max_image_dimensions: NonZeroUsize,
        embedding_size: NonZeroUsize,
        provider: ModelProviders,
    },
}

impl From<&AIModel> for Model {
    fn from(value: &AIModel) -> Self {
        match value {
            AIModel::AllMiniLML6V2 => Self::Text {
                supported_model: SupportedModels::AllMiniLML6V2,
                description: String::from("Sentence Transformer model, with 6 layers, version 2"),
                embedding_size: nonzero!(384usize),
                max_input_tokens: nonzero!(256usize),
                provider: ModelProviders::FastEmbed(FastEmbedProvider::new()),
            },
            AIModel::AllMiniLML12V2 => Self::Text {
                supported_model: SupportedModels::AllMiniLML12V2,
                description: String::from("Sentence Transformer model, with 12 layers, version 2."),
                embedding_size: nonzero!(384usize),
                // Token size source: https://huggingface.co/sentence-transformers/all-MiniLM-L12-v2#intended-uses
                max_input_tokens: nonzero!(256usize),
                provider: ModelProviders::FastEmbed(FastEmbedProvider::new()),
            },
            AIModel::BGEBaseEnV15 => Self::Text {
                supported_model: SupportedModels::BGEBaseEnV15,
                description: String::from(
                    "BAAI General Embedding model with English support, base scale, version 1.5.",
                ),
                embedding_size: nonzero!(768usize),
                // Token size source: https://huggingface.co/BAAI/bge-large-en/discussions/11#64e44de1623074ac850aa1ae
                max_input_tokens: nonzero!(512usize),
                provider: ModelProviders::FastEmbed(FastEmbedProvider::new()),
            },
            AIModel::BGELargeEnV15 => Self::Text {
                supported_model: SupportedModels::BGELargeEnV15,
                description: String::from(
                    "BAAI General Embedding model with English support, large scale, version 1.5.",
                ),
                embedding_size: nonzero!(1024usize),
                max_input_tokens: nonzero!(512usize),
                provider: ModelProviders::FastEmbed(FastEmbedProvider::new()),
            },
            AIModel::Resnet50 => Self::Image {
                supported_model: SupportedModels::Resnet50,
                description: String::from("Residual Networks model, with 50 layers."),
                embedding_size: nonzero!(2048usize),
                max_image_dimensions: nonzero!(224usize),
                provider: ModelProviders::FastEmbed(FastEmbedProvider::new()),
            },
            AIModel::ClipVitB32 => Self::Image {
                supported_model: SupportedModels::ClipVitB32,
                description: String::from(
                    "Contrastive Language-Image Pre-Training Vision transformer model, base scale.",
                ),
                embedding_size: nonzero!(512usize),
                max_image_dimensions: nonzero!(224usize),
                provider: ModelProviders::FastEmbed(FastEmbedProvider::new()),
            },
        }
    }
}

impl Model {
    #[tracing::instrument(skip(self))]
    pub fn embedding_size(&self) -> NonZeroUsize {
        match self {
            Model::Text { embedding_size, .. } => *embedding_size,
            Model::Image { embedding_size, .. } => *embedding_size,
        }
    }
    #[tracing::instrument(skip(self))]
    pub fn input_type(&self) -> AIStoreInputType {
        match self {
            Model::Text { .. } => AIStoreInputType::RawString,
            Model::Image { .. } => AIStoreInputType::Image,
        }
    }

    // TODO: model ndarray values is based on length of string or vec, so for now make sure strings
    // or vecs have different lengths
    #[tracing::instrument(skip(self))]
    pub fn model_ndarray(
        &self,
        storeinput: &StoreInput,
        action_type: InputAction,
    ) -> Result<StoreKey, AIProxyError> {
        let length = storeinput.len() as f32;
        if let (StoreInput::RawString(string), Model::Text { provider, .. }) = (storeinput, self) {
            return match provider {
                ModelProviders::FastEmbed(provider) => {
                    let embedding = provider.run_inference(string.clone().as_str());
                    Ok(StoreKey(<Array1<f32>>::from(embedding)))
                }
            };
        } else if let (StoreInput::Image(image), Model::Image { provider, .. }) = (storeinput, self)
        {
            // let embedding = provider.run_inference(image);
            // return StoreKey(embedding);
            Ok(StoreKey(
                Array1::from_iter(0..self.embedding_size().into()).mapv(|v| v as f32 * length),
            ))
        } else {
            return Err(AIProxyError::TypeMismatchError {
                model_type: storeinput.into(),
                input_type: self.into(),
                action_type,
            });
        }
    }

    #[tracing::instrument(skip(self))]
    pub fn max_input_token(&self) -> Option<NonZeroUsize> {
        match self {
            Model::Text {
                max_input_tokens, ..
            } => Some(*max_input_tokens),
            Model::Image { .. } => None,
        }
    }
    #[tracing::instrument(skip(self))]
    pub fn max_image_dimensions(&self) -> Option<NonZeroUsize> {
        match self {
            Model::Text { .. } => None,
            Model::Image {
                max_image_dimensions,
                ..
            } => Some(*max_image_dimensions),
        }
    }
    #[tracing::instrument(skip(self))]
    pub fn model_name(&self) -> String {
        match self {
            Model::Text {supported_model, ..} |
            Model::Image {supported_model, ..} => {
                supported_model.to_string()
            }
        }
    }
    #[tracing::instrument(skip(self))]
    pub fn model_description(&self) -> String {
        match self {
            Model::Text { description, .. } | Model::Image { description, .. } => {
                description.clone()
            }
        }
    }

    pub fn setup_provider(&mut self, supported_model: SupportedModels, cache_location: &PathBuf) {
        match self {
            Model::Text { provider, .. } | Model::Image { provider, .. } => match provider {
                ModelProviders::FastEmbed(provider) => {
                    provider.set_model(supported_model);
                    provider.set_cache_location(cache_location);
                }
            },
        }
    }

    pub fn load(&mut self) {
        match self {
            Model::Text { provider, .. } | Model::Image { provider, .. } => match provider {
                ModelProviders::FastEmbed(provider) => {
                    provider.load_model();
                }
            },
        }
    }

    pub fn get(&self) {
        match self {
            Model::Text { provider, .. } | Model::Image { provider, .. } => match provider {
                ModelProviders::FastEmbed(provider) => {
                    provider.get_model();
                }
            },
        }
    }
}

impl From<&Model> for AIStoreInputType {
    fn from(value: &Model) -> Self {
        match value {
            Model::Text { .. } => AIStoreInputType::RawString,
            Model::Image { .. } => AIStoreInputType::Image,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ModelInfo {
    name: String,
    input_type: AIStoreInputType,
    embedding_size: NonZeroUsize,
    max_input_tokens: Option<NonZeroUsize>,
    max_image_dimensions: Option<NonZeroUsize>,
    description: String,
}

impl ModelInfo {
    pub(crate) fn build(model: &Model) -> Self {
        Self {
            name: model.model_name(),
            input_type: model.input_type(),
            embedding_size: model.embedding_size(),
            max_input_tokens: model.max_input_token(),
            max_image_dimensions: model.max_image_dimensions(),
            description: model.model_description(),
        }
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum InputAction {
    Query,
    Index,
}

impl fmt::Display for InputAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Query => write!(f, "query"),
            Self::Index => write!(f, "index"),
        }
    }
}
