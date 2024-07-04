use ahnlich_types::keyval::StoreName;
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
}
