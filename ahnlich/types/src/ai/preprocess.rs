use serde::{Deserialize, Serialize};

/// The String input has to be tokenized before saving into the model.
/// The action to be performed if the string input is too larger than the maximum tokens a
/// model can take.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum StringAction {
    TruncateIfTokensExceed,
    ErrorIfTokensExceed,
}

/// The action to be performed if the image dimensions is larger than the maximum size a
/// model can take.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ImageAction {
    ResizeImage,
    ErrorIfDimensionsMismatch,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum PreprocessAction {
    RawString(StringAction),
    Image(ImageAction),
}
