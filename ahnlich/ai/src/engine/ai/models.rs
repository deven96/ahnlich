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
