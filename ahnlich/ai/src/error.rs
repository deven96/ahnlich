use ahnlich_types::{ai::AIStoreType, keyval::StoreName};
use thiserror::Error;

#[derive(Error, Debug, Eq, PartialEq, PartialOrd, Ord)]
pub enum AIProxyError {
    #[error("Store {0} not found")]
    StoreNotFound(StoreName),
    #[error("Store {0} already exists")]
    StoreAlreadyExists(StoreName),
    #[error("Proxy Errored with {0} ")]
    StandardError(String),
    #[error("Proxy Errored with {0} ")]
    DatabaseClientError(String),
    #[error("Reserved key {0} used")]
    ReservedError(String),
    #[error("Unexpected DB Response {0} ")]
    UnexpectedDBResponse(String),
    #[error("Store dimension is [{store_type}], input dimension of [{input_type}] was specified")]
    StoreTypeMismatch {
        store_type: AIStoreType,
        input_type: AIStoreType,
    },
}
