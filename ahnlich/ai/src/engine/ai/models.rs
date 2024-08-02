use crate::engine::ai::AIModelManager;
use ahnlich_types::{
    ai::{AIModel, AIStoreInputTypes},
    keyval::{StoreInput, StoreKey},
};
use ndarray::Array1;
use serde::{Deserialize, Serialize};
use std::num::NonZeroUsize;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModelInfo {
    pub name: String,
    pub embedding_size: usize,
    pub input_type: AIStoreInputTypes,
}

impl AIModelManager for AIModel {
    fn embedding_size(&self) -> NonZeroUsize {
        NonZeroUsize::new(self.model_info().embedding_size).expect("Failed to get embedding size")
    }
    // TODO: model ndarray values is based on length of string or vec, so for now make sure strings
    // or vecs have different lengths
    fn model_ndarray(&self, storeinput: &StoreInput) -> StoreKey {
        let length = storeinput.len() as f32;
        StoreKey(Array1::from_iter(0..self.model_info().embedding_size).mapv(|v| v as f32 * length))
    }

    fn model_info(&self) -> ModelInfo {
        match self {
            AIModel::Llama3 => ModelInfo {
                name: String::from("Llama3"),
                embedding_size: 100,
                input_type: AIStoreInputTypes::RawString,
            },
            AIModel::DALLE3 => ModelInfo {
                name: String::from("DALL.E 3"),
                embedding_size: 300,
                input_type: AIStoreInputTypes::Image,
            },
        }
    }
}
