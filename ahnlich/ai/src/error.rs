use ahnlich_types::{
    ai::{AIStoreInputType, PreprocessAction},
    keyval::StoreName,
};
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
    #[error("Cannot Query Using Input. Store expects [{store_query_model_type}], but input type [{storeinput_type}] was provided")]
    StoreQueryTypeMismatchError {
        store_query_model_type: AIStoreInputType,
        storeinput_type: AIStoreInputType,
    },
    #[error("Cannot Set Input. Store expects [{store_index_model_type}], input type [{storeinput_type}] was provided")]
    StoreSetTypeMismatchError {
        store_index_model_type: AIStoreInputType,
        storeinput_type: AIStoreInputType,
    },

    #[error("Max Token Exceeded. Model Expects [{max_token_size}], input type was [{input_token_size}] ")]
    TokenExceededError {
        max_token_size: usize,
        input_token_size: usize,
    },

    #[error(
        "Image Dimensions [{image_dimensions}] exceeds max model dimensions [{max_dimensions}] "
    )]
    ImageDimensionsMismatchError {
        image_dimensions: usize,
        max_dimensions: usize,
    },

    #[error("Used [{preprocess_action}] for [{input_type}] type")]
    PreprocessingMismatchError {
        input_type: AIStoreInputType,
        preprocess_action: PreprocessAction,
    },

    #[error("index_model or query_model not selected during aiproxy startup")]
    AIModelNotInitialized,
}
