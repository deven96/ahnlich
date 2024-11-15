use serde::{Deserialize, Serialize};
use std::fmt;

/// The String input has to be tokenized before saving into the model.
/// The action to be performed if the string input is too larger than the maximum tokens a
/// model can take.
#[derive(Copy, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StringAction {
    TruncateIfTokensExceed,
    ErrorIfTokensExceed,
    ModelPreprocessing
}

/// The action to be performed if the image dimensions is larger than the maximum size a
/// model can take.
#[derive(Copy, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImageAction {
    ResizeImage,
    ErrorIfDimensionsMismatch,
    ModelPreprocessing
}

#[derive(Copy, Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PreprocessAction {
    RawString(StringAction),
    Image(ImageAction),
}

impl fmt::Display for PreprocessAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RawString(_) => write!(f, "PreprocessString"),
            Self::Image(_) => write!(f, "PreprocessImage"),
        }
    }
}
