mod query;
mod server;

use std::{fmt, num::NonZeroUsize};

use ndarray::array;
pub use query::{AIQuery, AIServerQuery};
use serde::{Deserialize, Serialize};
pub use server::{AIServerResponse, AIServerResult, AIStoreInfo};

use crate::keyval::{StoreInput, StoreKey};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AIModel {
    Llama3,
}

impl AIModel {
    // TODO: Consider making this an option of nonzerousize
    pub fn embedding_size(&self) -> NonZeroUsize {
        NonZeroUsize::new(3).expect("Failed to get embedding size")
    }
    // TODO: model ndarray is based on length of string or vec, so for now make sure strings
    // or vecs have different lengths
    pub fn model_ndarray(&self, storeinput: &StoreInput) -> StoreKey {
        let length = storeinput.len() as f32;
        StoreKey(array![1.4, 2.5, 4.5].mapv(|v| v * length))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AIStoreType {
    RawString,
    Binary,
}

impl From<StoreInput> for AIStoreType {
    fn from(value: StoreInput) -> Self {
        match value {
            StoreInput::RawString(_) => AIStoreType::RawString,
            StoreInput::Binary(_) => AIStoreType::Binary,
        }
    }
}

impl fmt::Display for AIStoreType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RawString => write!(f, "{}", AIStoreType::RawString),
            Self::Binary => write!(f, "{}", AIStoreType::Binary),
        }
    }
}
