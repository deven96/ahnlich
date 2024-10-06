use ahnlich_types::{
    ai::{AIModel, AIStoreInputType},
    keyval::{StoreInput, StoreKey},
};
use ndarray::Array1;
use nonzero_ext::nonzero;
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;

pub enum Model {
    Text {
        name: String,
        description: String,
        embedding_size: NonZeroUsize,
        max_input_tokens: NonZeroUsize,
    },
    Image {
        name: String,
        description: String,
        max_image_dimensions: NonZeroUsize,
        embedding_size: NonZeroUsize,
    },
}

impl From<&AIModel> for Model {
    fn from(value: &AIModel) -> Self {
        match value {
            AIModel::Llama3 => Self::Text {
                name: String::from("Llama3"),
                description: String::from("Llama3, a text model"),
                embedding_size: nonzero!(100usize),
                max_input_tokens: nonzero!(100usize),
            },
            AIModel::DALLE3 => Self::Image {
                name: String::from("DALL.E 3"),
                description: String::from("Dalle3, an image model"),
                embedding_size: nonzero!(300usize),
                max_image_dimensions: nonzero!(300usize),
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
    pub fn input_type(&self) -> String {
        match self {
            Model::Text { .. } => AIStoreInputType::RawString.to_string(),
            Model::Image { .. } => AIStoreInputType::Image.to_string(),
        }
    }

    // TODO: model ndarray values is based on length of string or vec, so for now make sure strings
    // or vecs have different lengths
    #[tracing::instrument(skip(self))]
    pub fn model_ndarray(&self, storeinput: &StoreInput) -> StoreKey {
        let length = storeinput.len() as f32;
        StoreKey(Array1::from_iter(0..self.embedding_size().into()).mapv(|v| v as f32 * length))
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
            Model::Text { name, .. } => name.clone(),
            Model::Image { name, .. } => name.clone(),
        }
    }
    #[tracing::instrument(skip(self))]
    pub fn model_description(&self) -> String {
        match self {
            Model::Text { description, .. } => description.clone(),
            Model::Image { description, .. } => description.clone(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub(crate) struct ModelInfo {
    name: String,
    input_type: String,
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
