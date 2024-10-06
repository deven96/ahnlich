mod preprocess;
mod query;
mod server;
pub use preprocess::{ImageAction, PreprocessAction, StringAction};
pub use query::{AIQuery, AIServerQuery};
use serde::{Deserialize, Serialize};
pub use server::{AIServerResponse, AIServerResult, AIStoreInfo};
use std::fmt;

use crate::keyval::StoreInput;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AIModel {
    // Image model
    DALLE3,
    Llama3,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AIStoreInputType {
    RawString,
    Image,
}

impl From<&StoreInput> for AIStoreInputType {
    fn from(value: &StoreInput) -> Self {
        match value {
            StoreInput::RawString(_) => AIStoreInputType::RawString,
            StoreInput::Image(_) => AIStoreInputType::Image,
        }
    }
}

impl fmt::Display for AIStoreInputType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RawString => write!(f, "RawString"),
            Self::Image => write!(f, "Image"),
        }
    }
}
