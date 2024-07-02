mod query;
mod server;
use std::num::NonZeroUsize;

use serde::{Deserialize, Serialize};

pub use query::{AIQuery, AIServerQuery};
pub use server::{AIServerResponse, AIServerResult, AIStoreInfo};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AIModel {
    Llama3,
}

impl AIModel {
    pub fn embedding_size(&self) -> NonZeroUsize {
        todo!()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AIStoreType {
    RawString,
    Binary,
}
