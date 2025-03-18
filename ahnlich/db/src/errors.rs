use grpc_types::keyval::StoreName;

use fallible_collections::TryReserveError;
use grpc_types::algorithm::nonlinear::NonLinearAlgorithm;
use thiserror::Error;
use tonic::{Code, Status};

#[derive(Error, Debug, Eq, PartialEq)]
pub enum ServerError {
    #[error("Predicate {0} not found in store, attempt CREATEPREDINDEX with predicate")]
    PredicateNotFound(String),
    #[error("Non linear algorithm {0:?} not found in store, create store with support")]
    NonLinearIndexNotFound(NonLinearAlgorithm),
    #[error("Store {0:?} not found")]
    StoreNotFound(StoreName),
    #[error("Store {0:?} already exists")]
    StoreAlreadyExists(StoreName),
    #[error("Store dimension is [{store_dimension}], input dimension of [{input_dimension}] was specified")]
    StoreDimensionMismatch {
        store_dimension: usize,
        input_dimension: usize,
    },
    #[error("allocation error {0:?}")]
    Allocation(TryReserveError),
}

impl From<ServerError> for Status {
    fn from(input: ServerError) -> Status {
        let message = input.to_string();
        let code = match input {
            ServerError::StoreNotFound(_) => Code::NotFound,
            ServerError::StoreAlreadyExists(_) => Code::AlreadyExists,
            ServerError::StoreDimensionMismatch {
                store_dimension: _,
                input_dimension: _,
            } => Code::InvalidArgument,
            ServerError::PredicateNotFound(_) => Code::NotFound,
            ServerError::NonLinearIndexNotFound(_) => Code::NotFound,
            ServerError::Allocation(_) => Code::ResourceExhausted,
        };
        Status::new(code, message)
    }
}

impl From<TryReserveError> for ServerError {
    fn from(input: TryReserveError) -> Self {
        Self::Allocation(input)
    }
}
