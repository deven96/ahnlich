use crate::metadata::MetadataKey;
use crate::metadata::MetadataValue;
use ndarray::Array1;
use std::collections::HashMap as StdHashMap;

/// A store value for now is a simple key value pair of strings
pub type StoreValue = StdHashMap<MetadataKey, MetadataValue>;

/// A store key is always an f32 one dimensional array
#[derive(Debug, Clone)]
pub struct StoreKey(pub Array1<f32>);
