use thiserror::Error;
use tonic::{Code, Status};

#[derive(Error, Debug)]
pub enum AhnlichError {
    #[error("Invalid URI {0}")]
    InvalidURI(#[from] http::uri::InvalidUri),
    #[error("Transport issues with tonic {0}")]
    Tonic(#[from] tonic::transport::Error),
    #[error("Server error {0}")]
    ServerError(#[from] tonic::Status),
}

impl From<AhnlichError> for Status {
    fn from(input: AhnlichError) -> Status {
        let (code, message) = match input {
            AhnlichError::Tonic(err) => (Code::Internal, err.to_string()),
            AhnlichError::InvalidURI(_) => (Code::InvalidArgument, input.to_string()),
            AhnlichError::ServerError(a) => (a.code(), a.message().to_string()),
        };
        Status::new(code, message)
    }
}
