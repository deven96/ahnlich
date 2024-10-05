use thiserror::Error;

#[derive(Error, Debug, Eq, PartialEq)]
pub enum TypeError {
    #[error("Bytes could not be successfully decoded into an image.")]
    ImageBytesDecodeError,

    #[error("Image could not be resized.")]
    ImageResizeError,
}