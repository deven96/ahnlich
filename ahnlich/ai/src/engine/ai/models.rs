use crate::cli::server::SupportedModels;
use crate::engine::ai::providers::fastembed::FastEmbedProvider;
use crate::engine::ai::providers::ort::ORTProvider;
use crate::engine::ai::providers::ModelProviders;
use crate::engine::ai::providers::ProviderTrait;
use crate::error::AIProxyError;
use ahnlich_types::{
    ai::{AIModel, AIStoreInputType},
    keyval::{StoreInput, StoreKey},
};
use ndarray::{Array, Array1, Ix3};
use nonzero_ext::nonzero;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::num::NonZeroUsize;
use std::path::PathBuf;
use std::cmp::Ordering;
use image::{GenericImageView, ImageReader};
use std::io::Cursor;
use strum::Display;

#[derive(Display)]
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
        // width, height
        expected_image_dimensions: (NonZeroUsize, NonZeroUsize),
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
                expected_image_dimensions: (nonzero!(224usize), nonzero!(224usize)),
                provider: ModelProviders::ORT(ORTProvider::new()),
            },
            AIModel::ClipVitB32 => Self::Image {
                supported_model: SupportedModels::ClipVitB32,
                description: String::from(
                    "Contrastive Language-Image Pre-Training Vision transformer model, base scale.",
                ),
                embedding_size: nonzero!(512usize),
                expected_image_dimensions: (nonzero!(224usize), nonzero!(224usize)),
                provider: ModelProviders::ORT(ORTProvider::new()),
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
        storeinput: &ModelInput,
        action_type: &InputAction,
    ) -> Result<StoreKey, AIProxyError> {
        match self {
            Model::Text { provider, .. } | Model::Image { provider, .. } => {
                return match provider {
                    ModelProviders::FastEmbed(provider) => {
                        let embedding = provider.run_inference(storeinput, action_type);
                        Ok(StoreKey(<Array1<f32>>::from(embedding)))
                    },
                    ModelProviders::ORT(provider) => {
                        let embedding = provider.run_inference(storeinput, action_type);
                        Ok(StoreKey(<Array1<f32>>::from(embedding)))
                    }
                }
            }
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
    pub fn expected_image_dimensions(&self) -> Option<(NonZeroUsize, NonZeroUsize)> {
        match self {
            Model::Text { .. } => None,
            Model::Image {
                expected_image_dimensions: (width, height),
                ..
            } => Some((*width, *height)),
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

    pub fn setup_provider(&mut self, supported_model: &SupportedModels, cache_location: &PathBuf) {
        match self {
            Model::Text { provider, .. } | Model::Image { provider, .. } => match provider {
                ModelProviders::FastEmbed(provider) => {
                    provider.set_model(supported_model);
                    provider.set_cache_location(cache_location);
                },
                ModelProviders::ORT(provider) => {
                    provider.set_model(supported_model);
                    provider.set_cache_location(cache_location);
                },
            },
        }
    }

    pub fn load(&mut self) {
        match self {
            Model::Text { provider, .. } | Model::Image { provider, .. } => match provider {
                ModelProviders::FastEmbed(provider) => {
                    provider.load_model();
                },
                ModelProviders::ORT(provider) => {
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
                },
                ModelProviders::ORT(provider) => {
                    provider.get_model();
                },
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
    expected_image_dimensions: Option<(NonZeroUsize, NonZeroUsize)>,
    description: String,
}

impl ModelInfo {
    pub(crate) fn build(model: &Model) -> Self {
        Self {
            name: model.model_name(),
            input_type: model.input_type(),
            embedding_size: model.embedding_size(),
            max_input_tokens: model.max_input_token(),
            expected_image_dimensions: model.expected_image_dimensions(),
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

#[derive(Debug)]
pub enum ModelInput {
    Text(String),
    Image(ImageArray),
}


#[derive(Debug, Clone, PartialEq, Hash, Eq)]
pub struct ImageArray {
    array: Array<u8, Ix3>,
    bytes: Vec<u8>
}

impl ImageArray {
    pub fn try_new(bytes: Vec<u8>) -> Result<Self, AIProxyError> {
        let img_reader = ImageReader::new(Cursor::new(&bytes))
            .with_guessed_format()
            .map_err(|_| AIProxyError::ImageBytesDecodeError)?;

        let img = img_reader
            .decode()
            .map_err(|_| AIProxyError::ImageBytesDecodeError)?;
        let (width, height) = img.dimensions();
        let channels = img.color().channel_count();
        let shape = (height as usize, width as usize, channels as usize);
        let array = Array::from_shape_vec(shape, img.into_bytes())
            .map_err(|_| AIProxyError::ImageBytesDecodeError)?;
        Ok(ImageArray { array, bytes })
    }

    pub fn get_array(&self) -> &Array<u8, Ix3> {
        &self.array
    }

    pub fn get_bytes(&self) -> &Vec<u8> {
        &self.bytes
    }

    pub fn resize(&self, width: usize, height: usize) -> Result<Self, AIProxyError> {
        let img_reader = ImageReader::new(Cursor::new(&self.bytes))
            .with_guessed_format()
            .map_err(|_| AIProxyError::ImageBytesDecodeError)?;
        let img_format = img_reader.format().ok_or(AIProxyError::ImageBytesDecodeError)?;
        let original_img = img_reader
            .decode()
            .map_err(|_| AIProxyError::ImageBytesDecodeError)?;

        let resized_img = original_img.resize_exact(width as u32, height as u32,
                                                    image::imageops::FilterType::Triangle);
        let channels = resized_img.color().channel_count();
        let shape = (height as usize, width as usize, channels as usize);

        let mut buffer = Cursor::new(Vec::new());
        resized_img.write_to(&mut buffer, img_format)
            .map_err(|_| AIProxyError::ImageResizeError)?;

        let flattened_pixels = resized_img.into_bytes();
        let array = Array::from_shape_vec(shape, flattened_pixels)
            .map_err(|_| AIProxyError::ImageResizeError)?;
        let bytes = buffer.into_inner();
        Ok(ImageArray { array, bytes })
    }

    pub fn image_dim(&self) -> (NonZeroUsize, NonZeroUsize) {
        let shape = self.array.shape();
        return (NonZeroUsize::new(shape[1]).unwrap(),
                NonZeroUsize::new(shape[0]).unwrap()); // (width, height)
    }
}

impl Serialize for ImageArray {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bytes(self.get_bytes())
    }
}

impl<'de> Deserialize<'de> for ImageArray {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = Deserialize::deserialize(deserializer)?;
        Ok(ImageArray::try_new(bytes).map_err(serde::de::Error::custom)?)
    }
}

impl Ord for ImageArray {
    fn cmp(&self, other: &Self) -> Ordering {
        let (array_vec, _) = self.array.clone().into_raw_vec_and_offset();
        let (other_vec, _) = other.array.clone().into_raw_vec_and_offset();
        array_vec.cmp(&other_vec)
    }
}

impl PartialOrd for ImageArray {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl From<StoreInput> for ModelInput {
    fn from(value: StoreInput) -> Self {
        match value {
            StoreInput::RawString(s) => ModelInput::Text(s),
            StoreInput::Image(bytes) => ModelInput::Image(ImageArray::try_new(bytes).unwrap()),
        }
    }
}

impl Into<AIStoreInputType> for &ModelInput {
    fn into(self) -> AIStoreInputType {
        match self {
            ModelInput::Text(_) => AIStoreInputType::RawString,
            ModelInput::Image(_) => AIStoreInputType::Image,
        }
    }
}
