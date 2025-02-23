use ahnlich_types::bincode::BincodeSerError;
use fallible_collections::TryReserveError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum AhnlichError {
    #[error("Invalid URI {0}")]
    InvalidURI(#[from] http::uri::InvalidUri),
    #[error("Transport issues with tonic {0}")]
    Tonic(#[from] tonic::transport::Error),
    #[error("Server error {0}")]
    ServerError(#[from] tonic::Status),
    #[error("std io error {0}")]
    Standard(#[from] std::io::Error),
    #[error("{0}")]
    BinCodeSerAndDeser(#[from] BincodeSerError),
    #[error("allocation error {0:?}")]
    Allocation(TryReserveError),
    #[error("bincode deserialize error {0}")]
    Bincode(#[from] bincode::Error),
    #[error("db error {0}")]
    DbError(String),
    #[error("empty response")]
    EmptyResponse,
    #[error("deadpool error {0}")]
    PoolError(String),
    #[error("ai proxy error {0}")]
    AIProxyError(String),
}

impl<E: std::fmt::Debug> From<deadpool::managed::PoolError<E>> for AhnlichError {
    fn from(input: deadpool::managed::PoolError<E>) -> Self {
        Self::PoolError(format!("{input:?}"))
    }
}

impl From<deadpool::managed::BuildError> for AhnlichError {
    fn from(input: deadpool::managed::BuildError) -> Self {
        Self::PoolError(format!("{input}"))
    }
}

impl From<TryReserveError> for AhnlichError {
    fn from(input: TryReserveError) -> Self {
        Self::Allocation(input)
    }
}
