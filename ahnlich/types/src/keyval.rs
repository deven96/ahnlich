use crate::metadata::MetadataKey;
use crate::metadata::MetadataValue;
use ndarray::Array1;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap as StdHashMap;

/// Name of a Store
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StoreName(pub String);

/// A store value for now is a simple key value pair of strings
pub type StoreValue = StdHashMap<MetadataKey, MetadataValue>;

/// A store key is always an f64 one dimensional array
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StoreKey(pub Array1<f64>);

impl Eq for StoreKey {}

impl PartialEq for StoreKey {
    fn eq(&self, other: &Self) -> bool {
        if self.0.shape() != other.0.shape() {
            return false;
        }
        self.0
            .iter()
            .zip(other.0.iter())
            .all(|(x, y)| (x - y).abs() < std::f64::EPSILON)
    }
}
