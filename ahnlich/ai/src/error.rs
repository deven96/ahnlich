use fallible_collections::TryReserveError;
use grpc_types::{
    ai::{models::AiStoreInputType, preprocess::PreprocessAction},
    keyval::StoreName,
};
use thiserror::Error;
use tokio::sync::oneshot::error::RecvError;
use tonic::{Code, Status};

use crate::engine::ai::models::InputAction;

#[derive(Error, Clone, Debug, PartialEq, Eq)]
pub enum AIProxyError {
    #[error("Store {0:?} not found")]
    StoreNotFound(StoreName),
    #[error("Store {0:?} already exists")]
    StoreAlreadyExists(StoreName),

    #[error("{0} input not specified")]
    InputNotSpecified(String),

    #[error("Proxy Errored with {0} ")]
    DatabaseClientError(String),
    #[error("Reserved key {0} used")]
    ReservedError(String),
    #[error("Unexpected DB Response {0} ")]
    UnexpectedDBResponse(String),
    #[error("Cannot {action} Input. Store expects [{index_model_type:?}], input type [{storeinput_type:?}] was provided")]
    StoreTypeMismatchError {
        action: InputAction,
        index_model_type: AiStoreInputType,
        storeinput_type: AiStoreInputType,
    },

    #[error("Max Token Exceeded. Model Expects [{max_token_size}], input type was [{input_token_size}].")]
    TokenExceededError {
        max_token_size: usize,
        input_token_size: usize,
    },

    #[error("Model preprocessing for {model_name} failed: {message}.")]
    ModelPreprocessingError { model_name: String, message: String },

    #[error("Model postprocessing for {model_name} failed: {message}.")]
    ModelPostprocessingError { model_name: String, message: String },

    #[error("Pooling operation failed: {message}.")]
    PoolingError { message: String },

    #[error(
        "Image Dimensions [({0}, {1})] does not match the expected model dimensions [({2}, {3})]",
        image_dimensions.0, image_dimensions.1, expected_dimensions.0, expected_dimensions.1
    )]
    ImageDimensionsMismatchError {
        image_dimensions: (usize, usize),
        expected_dimensions: (usize, usize),
    },
    #[error("API Builder Error: {0}")]
    APIBuilderError(String),
    #[error("ORT Error {0}")]
    ORTError(String),
    #[error("Used [{preprocess_action:?}] for [{input_type:?}] type")]
    PreprocessingMismatchError {
        input_type: AiStoreInputType,
        preprocess_action: PreprocessAction,
    },

    #[error("Unknown enum value {0}")]
    UnknownEnumValue(#[from] prost::UnknownEnumValue),

    #[error("index_model or query_model not selected or loaded during aiproxy startup")]
    AIModelNotInitialized,

    #[error("index_model or query_model [{model_name}] not supported")]
    AIModelNotSupported { model_name: String },

    #[error("Vector normalization error: [{message}]")]
    VectorNormalizationError { message: String },

    #[error("Image normalization error: [{message}]")]
    ImageNormalizationError { message: String },

    #[error("ImageArray to NdArray conversion error: [{message}]")]
    ImageArrayToNdArrayError { message: String },

    #[error("Onnx output transform error: [{message}]")]
    OnnxOutputTransformError { message: String },

    #[error("Rescale error: [{message}]")]
    RescaleError { message: String },

    #[error("Center crop error: [{message}]")]
    CenterCropError { message: String },

    // TODO: Add SendError from mpsc::Sender into this variant
    #[error("Error sending request to model thread")]
    AIModelThreadSendError,

    #[error("Error receiving response from model thread")]
    AIModelRecvError(#[from] RecvError),

    #[error("Dimensions Mismatch between index [{index_model_dim}], and Query [{query_model_dim}] Models.")]
    DimensionsMismatchError {
        index_model_dim: usize,
        query_model_dim: usize,
    },
    #[error("allocation error {0:?}")]
    Allocation(TryReserveError),

    #[error("Error initializing a model thread {0}.")]
    ModelInitializationError(String),

    #[error("Bytes could not be successfully decoded into an image.")]
    ImageBytesDecodeError,

    #[error("Image could not be successfully encoded into bytes.")]
    ImageBytesEncodeError,

    #[error(
        "Image can't have zero value in any dimension. Found height: {height}, width: {width}"
    )]
    ImageNonzeroDimensionError { width: usize, height: usize },

    #[error("Image could not be resized. {0}")]
    ImageResizeError(String),

    #[error("Image could not be cropped.")]
    ImageCropError,

    #[error("Model provider failed on preprocessing the input {0}")]
    ModelProviderPreprocessingError(String),

    #[error("Model provider failed on running inference {0}")]
    ModelProviderRunInferenceError(String),

    #[error("Model provider failed on postprocessing the output {0}")]
    ModelProviderPostprocessingError(String),

    #[error("Tokenize error: {message}")]
    ModelTokenizationError { message: String },

    #[error("Cannot call DelKey on store with `store_original` as false")]
    DelKeyError,

    #[error("Tokenizer for model failed to load: {message}")]
    ModelTokenizerLoadError { message: String },

    #[error("Unable to load config: [{message}].")]
    ModelConfigLoadError { message: String },
}

impl From<TryReserveError> for AIProxyError {
    fn from(input: TryReserveError) -> Self {
        Self::Allocation(input)
    }
}

impl From<ort::Error> for AIProxyError {
    fn from(input: ort::Error) -> Self {
        Self::ORTError(input.to_string())
    }
}

impl From<AIProxyError> for Status {
    fn from(input: AIProxyError) -> Status {
        let message = input.to_string();
        let code = match input {
            AIProxyError::StoreNotFound(_) => Code::NotFound,
            AIProxyError::StoreAlreadyExists(_) => Code::AlreadyExists,
            AIProxyError::DatabaseClientError(_)
            | AIProxyError::UnexpectedDBResponse(_)
            | AIProxyError::DelKeyError => Code::FailedPrecondition,
            AIProxyError::ReservedError(_)
            | AIProxyError::StoreTypeMismatchError {
                action: _,
                index_model_type: _,
                storeinput_type: _,
            }
            | AIProxyError::ImageDimensionsMismatchError {
                image_dimensions: _,
                expected_dimensions: _,
            }
            | AIProxyError::DimensionsMismatchError {
                index_model_dim: _,
                query_model_dim: _,
            }
            | AIProxyError::PreprocessingMismatchError {
                input_type: _,
                preprocess_action: _,
            }
            | AIProxyError::UnknownEnumValue(_)
            | AIProxyError::InputNotSpecified(_) => Code::InvalidArgument,
            AIProxyError::TokenExceededError {
                max_token_size: _,
                input_token_size: _,
            } => Code::OutOfRange,
            _others => Code::Internal,
        };
        Status::new(code, message)
    }
}
