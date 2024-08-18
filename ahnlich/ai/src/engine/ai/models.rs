use ahnlich_types::{
    ai::{AIModel, AIStoreInputType},
    keyval::{StoreInput, StoreKey},
};
use ndarray::Array1;
use nonzero_ext::nonzero;
use std::num::NonZeroUsize;

pub enum Model {
    Text {
        name: String,
        embedding_size: NonZeroUsize,
        max_input_tokens: NonZeroUsize,
    },
    Image {
        name: String,
        max_image_dimensions: NonZeroUsize,
        embedding_size: NonZeroUsize,
    },
}

impl From<&AIModel> for Model {
    fn from(value: &AIModel) -> Self {
        match value {
            AIModel::Llama3 => Self::Text {
                name: String::from("Llama3"),
                embedding_size: nonzero!(100usize),
                max_input_tokens: nonzero!(100usize),
            },
            AIModel::DALLE3 => Self::Image {
                name: String::from("DALL.E 3"),
                embedding_size: nonzero!(300usize),
                max_image_dimensions: nonzero!(300usize),
            },
        }
    }
}

impl Model {
    pub fn embedding_size(&self) -> NonZeroUsize {
        match self {
            Model::Text { embedding_size, .. } => *embedding_size,
            Model::Image { embedding_size, .. } => *embedding_size,
        }
    }
    pub fn input_type(&self) -> String {
        match self {
            Model::Text { .. } => AIStoreInputType::RawString.to_string(),
            Model::Image { .. } => AIStoreInputType::Image.to_string(),
        }
    }

    // TODO: model ndarray values is based on length of string or vec, so for now make sure strings
    // or vecs have different lengths
    pub fn model_ndarray(&self, storeinput: &StoreInput) -> StoreKey {
        let length = storeinput.len() as f32;
        StoreKey(Array1::from_iter(0..self.embedding_size().into()).mapv(|v| v as f32 * length))
    }

    pub fn max_input_token(&self) -> Option<NonZeroUsize> {
        match self {
            Model::Text {
                max_input_tokens, ..
            } => Some(*max_input_tokens),
            Model::Image { .. } => None,
        }
    }
    pub fn max_image_dimensions(&self) -> Option<NonZeroUsize> {
        match self {
            Model::Text { .. } => None,
            Model::Image {
                max_image_dimensions,
                ..
            } => Some(*max_image_dimensions),
        }
    }
}
