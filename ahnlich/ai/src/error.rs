use ahnlich_types::{
    ai::{AIStoreInputType, PreprocessAction},
    keyval::StoreName,
};
use fallible_collections::TryReserveError;
use thiserror::Error;
use tokio::sync::oneshot::error::RecvError;

use crate::engine::ai::models::InputAction;

#[derive(Error, Debug, PartialEq, Eq)]
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
    #[error("Cannot {action} Input. Store expects [{index_model_type}], input type [{storeinput_type}] was provided")]
    StoreTypeMismatchError {
        action: InputAction,
        index_model_type: AIStoreInputType,
        storeinput_type: AIStoreInputType,
    },
    #[error("Shape error {0}")]
    EmbeddingShapeError(String),
    #[error("Max Token Exceeded. Model Expects [{max_token_size}], input type was [{input_token_size}] ")]
    TokenExceededError {
        max_token_size: usize,
        input_token_size: usize,
    },

    #[error("Model does not support token truncation.")]
    TokenTruncationNotSupported,

    #[error(
        "Image Dimensions [({0}, {1})] does not match the expected model dimensions [({2}, {3})]",
        image_dimensions.0, image_dimensions.1, expected_dimensions.0, expected_dimensions.1
    )]
    ImageDimensionsMismatchError {
        image_dimensions: (usize, usize),
        expected_dimensions: (usize, usize),
    },
    #[error("Error initializing text embedding")]
    TextEmbeddingInitError(String),
    #[error("API Builder Error {0}")]
    APIBuilderError(String),
    #[error("Tokenizer initialization error {0}")]
    TokenizerInitError(String),
    #[error("ORT Error {0}")]
    ORTError(String),
    #[error("Used [{preprocess_action}] for [{input_type}] type")]
    PreprocessingMismatchError {
        input_type: AIStoreInputType,
        preprocess_action: PreprocessAction,
    },

    #[error("index_model or query_model not selected or loaded during aiproxy startup")]
    AIModelNotInitialized,

    #[error("Cache location for model was not initialized")]
    CacheLocationNotInitiailized,

    #[error("index_model or query_model not supported")]
    AIModelNotSupported,

    // TODO: Add SendError from mpsc::Sender into this variant
    #[error("Error sending request to model thread")]
    AIModelThreadSendError,

    #[error("Error receiving response from model thread")]
    AIModelRecvError(#[from] RecvError),

    #[error("Dimensions Mismatch between index [{index_model_dim}], and Query [{query_model_dim}] Models")]
    DimensionsMismatchError {
        index_model_dim: usize,
        query_model_dim: usize,
    },
    #[error("allocation error {0:?}")]
    Allocation(TryReserveError),

    #[error("Error initializing a model thread {0}")]
    ModelInitializationError(String),

    #[error("Bytes could not be successfully decoded into an image.")]
    ImageBytesDecodeError,

    #[error(
        "Image can't have zero value in any dimension. Found height: {height}, width: {width}"
    )]
    ImageNonzeroDimensionError { width: usize, height: usize },

    #[error("Image could not be resized.")]
    ImageResizeError,

    #[error("Model provider failed on preprocessing the input.")]
    ModelProviderPreprocessingError,

    #[error("Model provider failed on running inference.")]
    ModelProviderRunInferenceError,

    #[error("Model provider failed on postprocessing the output.")]
    ModelProviderPostprocessingError,

    #[error("Model provider failed on tokenization of text inputs.")]
    ModelTokenizationError,

    #[error("Cannot call DelKey on store with `store_original` as false")]
    DelKeyError,
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
