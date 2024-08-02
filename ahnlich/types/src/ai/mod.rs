mod preprocess;
mod query;
mod server;

use std::{fmt, num::NonZeroUsize};

use ndarray::Array1;
pub use preprocess::{ImageAction, PreprocessAction, StringAction};
pub use query::{AIQuery, AIServerQuery};
use serde::{Deserialize, Serialize};
pub use server::{AIServerResponse, AIServerResult, AIStoreInfo};

use crate::keyval::{StoreInput, StoreKey};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AIModel {
    // Image model
    DALLE3,
    Llama3,
}

impl AIModel {
    // TODO: Consider making this an option of nonzerousize
    pub fn embedding_size(&self) -> NonZeroUsize {
        NonZeroUsize::new(self.model_info().embedding_size).expect("Failed to get embedding size")
    }
    // TODO: model ndarray is based on length of string or vec, so for now make sure strings
    // or vecs have different lengths
    pub fn model_ndarray(&self, storeinput: &StoreInput) -> StoreKey {
        let length = storeinput.len() as f32;
        StoreKey(Array1::from_iter(0..self.model_info().embedding_size).mapv(|v| v as f32 * length))
    }

    pub fn model_info(&self) -> AIModelInfo {
        match self {
            AIModel::Llama3 => AIModelInfo {
                embedding_size: 100,
                input_type: AIStoreInputTypes::RawString,
            },
            AIModel::DALLE3 => AIModelInfo {
                embedding_size: 300,
                input_type: AIStoreInputTypes::Image,
            },
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AIModelInfo {
    pub embedding_size: usize,
    pub input_type: AIStoreInputTypes,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AIStoreInputTypes {
    RawString,
    Image,
}

impl From<StoreInput> for AIStoreInputTypes {
    fn from(value: StoreInput) -> Self {
        match value {
            StoreInput::RawString(_) => AIStoreInputTypes::RawString,
            StoreInput::Image(_) => AIStoreInputTypes::Image,
        }
    }
}

impl fmt::Display for AIStoreInputTypes {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RawString => write!(f, "RawString"),
            Self::Image => write!(f, "Image"),
        }
    }
}
