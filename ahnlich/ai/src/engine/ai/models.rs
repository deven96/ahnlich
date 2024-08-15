use crate::engine::ai::AIModelManager;
use ahnlich_types::{
    ai::{AIModel, AIStoreInputType},
    keyval::{StoreInput, StoreKey},
};
use ndarray::Array1;
use nonzero_ext::nonzero;
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelInfo {
    pub name: String,
    pub embedding_size: NonZeroUsize,
    pub input_type: AIStoreInputType,
    pub max_token: NonZeroUsize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct TextModelInfo {
    pub name: String,
    pub embedding_size: NonZeroUsize,
    pub input_type: AIStoreInputType,
    pub max_token: NonZeroUsize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MediaModelInfo {
    pub name: String,
    pub embedding_size: NonZeroUsize,
    pub input_type: AIStoreInputType,
}

impl AIModelManager for AIModel {
    fn embedding_size(&self) -> NonZeroUsize {
        self.model_info().embedding_size
    }
    // TODO: model ndarray values is based on length of string or vec, so for now make sure strings
    // or vecs have different lengths
    fn model_ndarray(&self, storeinput: &StoreInput) -> StoreKey {
        let length = storeinput.len() as f32;
        StoreKey(
            Array1::from_iter(0..self.model_info().embedding_size.into())
                .mapv(|v| v as f32 * length),
        )
    }

    fn model_info(&self) -> ModelInfo {
        match self {
            AIModel::Llama3 => ModelInfo {
                name: String::from("Llama3"),
                embedding_size: nonzero!(100usize),
                input_type: AIStoreInputType::RawString,
                max_token: nonzero!(100usize),
            },
            AIModel::DALLE3 => ModelInfo {
                name: String::from("DALL.E 3"),
                embedding_size: nonzero!(300usize),
                input_type: AIStoreInputType::Image,
                max_token: nonzero!(300usize),
            },
        }
    }
}

pub(crate) enum Model {
    Text {
        name: String,
        embedding_size: NonZeroUsize,
        input_type: AIStoreInputType,
        max_input_tokens: NonZeroUsize,
    },
    Image {
        name: String,
        max_image_dimensions: NonZeroUsize,
        input_type: AIStoreInputType,
        embedding_size: NonZeroUsize,
    },
}

impl From<&AIModel> for Model {
    fn from(value: &AIModel) -> Self {
        match value {
            AIModel::Llama3 => Self::Text {
                name: String::from("Llama3"),
                input_type: AIStoreInputType::RawString,
                embedding_size: nonzero!(100usize),
                max_input_tokens: nonzero!(100usize),
            },
            AIModel::DALLE3 => Self::Image {
                name: String::from("DALL.E 3"),
                input_type: AIStoreInputType::Image,
                embedding_size: nonzero!(300usize),
                max_image_dimensions: nonzero!(300usize),
            },
        }
    }
}

impl Model {
    pub(crate) fn embedding_size(&self) -> NonZeroUsize {
        match self {
            Model::Text { embedding_size, .. } => *embedding_size,
            Model::Image { embedding_size, .. } => *embedding_size,
        }
    }
    pub(crate) fn input_type(&self) -> AIStoreInputType {
        match self {
            Model::Text { input_type, .. } => input_type.clone(),
            Model::Image { input_type, .. } => input_type.clone(),
        }
    }
}
