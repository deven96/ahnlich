use crate::metadata::MetadataKey;
use crate::metadata::MetadataValue;
use ndarray::Array1;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap as StdHashMap;
use std::fmt;

/// Name of a Store
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StoreName(pub String);

impl fmt::Display for StoreName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A store value for now is a simple key value pair of strings
pub type StoreValue = StdHashMap<MetadataKey, MetadataValue>;

/// A store key is always an f32 one dimensional array
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StoreKey(pub Array1<f32>);

impl StoreKey {
    pub fn dimension(&self) -> usize {
        self.0.len()
    }
}

impl Eq for StoreKey {}

impl PartialEq for StoreKey {
    fn eq(&self, other: &Self) -> bool {
        if self.0.shape() != other.0.shape() {
            return false;
        }
        // std::f32::EPSILON adheres to the IEEE 754 standard and we use it here to determine when
        // two Array1<f32> are extremely similar to the point where the differences are neglible.
        // We can modify to allow for greater precision, however we currently only
        // use it for PartialEq and not for it's distinctive properties. For that, within the
        // server we defer to using StoreKeyId whenever we want to compare distinctive Array1<f32>
        self.0
            .iter()
            .zip(other.0.iter())
            .all(|(x, y)| (x - y).abs() < std::f32::EPSILON)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StoreInput {
    RawString(String),
    Binary(Vec<u8>),
}

impl From<StoreInput> for MetadataValue {
    fn from(value: StoreInput) -> Self {
        match value {
            StoreInput::Binary(binary) => MetadataValue::Binary(binary),
            StoreInput::RawString(s) => MetadataValue::RawString(s),
        }
    }
}

impl From<MetadataValue> for StoreInput {
    fn from(value: MetadataValue) -> Self {
        match value {
            MetadataValue::Binary(binary) => StoreInput::Binary(binary),
            MetadataValue::RawString(s) => StoreInput::RawString(s),
        }
    }
}
