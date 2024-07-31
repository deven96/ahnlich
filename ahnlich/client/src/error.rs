use thiserror::Error;

#[derive(Error, Debug)]
pub enum AhnlichError {
    #[error("std io error {0}")]
    Standard(#[from] std::io::Error),
    #[error("bincode serialize error {0}")]
    BinCode(#[from] bincode::Error),
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
