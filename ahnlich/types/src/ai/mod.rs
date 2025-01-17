mod preprocess;
mod query;
mod server;
pub use preprocess::PreprocessAction;
pub use query::{AIQuery, AIServerQuery};
use serde::{Deserialize, Serialize};
pub use server::{AIServerResponse, AIServerResult, AIStoreInfo};
use std::fmt;

use crate::keyval::StoreInput;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AIModel {
    AllMiniLML6V2,
    AllMiniLML12V2,
    BGEBaseEnV15,
    BGELargeEnV15,
    Resnet50,
    ClipVitB32Image,
    ClipVitB32Text,
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

#[derive(Serialize, Deserialize, Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Hash, Ord)]
#[allow(clippy::upper_case_acronyms)]
// list of execution providers to attempt to use
// Considered safe to initialize with `ExecutionProvider::to_provider()` as any unavailable execution
// provider fails "silenty" but can be viewed with `RUST_LOG='ort=debug'`
// https://ort.pyke.io/perf/execution-providers
//
// If provided execution provider cannot be initialized, then this fails
pub enum ExecutionProvider {
    TensorRT,
    CUDA,
    DirectML,
    CoreML,
}
