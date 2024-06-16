use thiserror::Error;
use types::keyval::StoreName;
use types::metadata::MetadataKey;

/// TODO: Move to shared rust types so library can deserialize it from the TCP response
#[derive(Error, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ServerError {
    #[error("Predicate {0} not found in store, attempt CREATEINDEX with predicate")]
    PredicateNotFound(MetadataKey),
    #[error("Store {0} not found")]
    StoreNotFound(StoreName),
    #[error("Store {0} already exists")]
    StoreAlreadyExists(StoreName),
    #[error("Store dimension is [{store_dimension}], input dimension of [{input_dimension}] was specified")]
    StoreDimensionMismatch {
        store_dimension: usize,
        input_dimension: usize,
    },
    #[error("Could not deserialize query, error is {0}")]
    QueryDeserializeError(String),
    #[error("Could not load store snapshot: {0}")]
    SnapshotLoadError(String),
}
