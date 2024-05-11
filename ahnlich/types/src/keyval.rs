use crate::metadata::MetadataKey;
use crate::metadata::MetadataValue;
use ndarray::Array1;
use std::collections::HashMap as StdHashMap;
use std::ops::Deref;

/// Name of a Store
#[derive(Debug, Clone, PartialEq, Eq, Ord, PartialOrd, Hash)]
pub struct StoreName(pub String);

/// A store value for now is a simple key value pair of strings
pub type StoreValue = StdHashMap<MetadataKey, MetadataValue>;

/// A store key is always an f64 one dimensional array
#[derive(Debug, Clone, PartialEq)]
pub struct StoreKey(pub Array1<f64>);

/// Search input is just also an f64 one dimensional array
pub type SearchInput = Array1<f64>;
