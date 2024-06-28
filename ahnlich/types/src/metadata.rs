use serde::Deserialize;
use serde::Serialize;
use std::fmt;

/// New types for store metadata key and values
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub enum MetadataKey {
    RawString(String),
    Binary(Vec<u8>),
}

impl fmt::Display for MetadataKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::RawString(s) => write!(f, "{}", s),
            Self::Binary(b) => write!(f, "{:?}", b),
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MetadataValue(String);
impl MetadataValue {
    pub fn new(input: String) -> Self {
        Self(input)
    }
}
