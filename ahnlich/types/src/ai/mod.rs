mod query;
mod server;

use serde::{Deserialize, Serialize};

pub use query::{AIQuery, AIServerQuery};
pub use server::{AIServerResponse, AIServerResult, AIStoreInfo};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AIModel {
    Llama3,
}

impl AIModel {
    pub fn embedding_size(&self) -> usize {
        todo!()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AIStoreType {
    RawString,
    Binary,
}
