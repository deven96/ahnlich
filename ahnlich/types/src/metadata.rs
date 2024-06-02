use serde::Deserialize;
use serde::Serialize;
use std::fmt;
/// New types for store metadata key and values
#[derive(Debug, Clone, Hash, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(transparent)]
pub struct MetadataKey(String);
impl MetadataKey {
    pub fn new(input: String) -> Self {
        Self(input)
    }
}

impl fmt::Display for MetadataKey {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
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
