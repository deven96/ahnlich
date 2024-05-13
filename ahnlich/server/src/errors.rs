use types::keyval::StoreName;
use types::metadata::MetadataKey;

/// TODO: Move to shared rust types so library can deserialize it from the TCP response
#[derive(Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum ServerError {
    PredicateNotFound(MetadataKey),
    StoreNotFound(StoreName),
    StoreAlreadyExists(StoreName),
    StoreDimensionMismatch {
        store_dimension: usize,
        input_dimension: usize,
    },
}
