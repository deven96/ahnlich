mod query;
mod server;

use std::{fmt, num::NonZeroUsize};

use ndarray::array;
pub use query::{AIQuery, AIServerQuery};
use rand::Rng;
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

    pub fn model_ndarray(&self, _storeinput: &StoreInput) -> StoreKey {
        let mut rng = rand::thread_rng();
        StoreKey(array![1.4, 1.5, 0.5].mapv(|v| v * rng.gen_range(0.2..2.1)))
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
