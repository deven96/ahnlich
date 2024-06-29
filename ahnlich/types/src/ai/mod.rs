mod query;
mod server;

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AIModel {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum AIStoreType {
    RawString,
    Binary,
}
