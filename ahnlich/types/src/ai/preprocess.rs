use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Copy, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PreprocessAction {
    NoPreprocessing,
    ModelPreprocessing
}

impl fmt::Display for PreprocessAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoPreprocessing => write!(f, "NoPreprocessing"),
            Self::ModelPreprocessing => write!(f, "ModelPreprocessing"),
        }
    }
}
