use thiserror::Error;

#[derive(Error, Debug)]
pub enum AhnlichError {
    #[error("std io error {0}")]
    Standard(#[from] std::io::Error),
    #[error("bincode serialize error {0}")]
    BinCode(#[from] bincode::Error),
    #[error("db error")]
    DbError(String),
    #[error("empty response")]
    EmptyResponse,
    #[error("r2d2 pool error")]
    PoolError(#[from] r2d2::Error),
}
