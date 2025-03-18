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
        let message = input.to_string();
        let code = match input {
            AhnlichError::Tonic(_) => Code::Internal,
            AhnlichError::InvalidURI(_) => Code::InvalidArgument,
            AhnlichError::ServerError(a) => a.code(),
        };
        Status::new(code, message)
    }
}
