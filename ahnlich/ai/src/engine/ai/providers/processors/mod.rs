use crate::engine::ai::models::ImageArray;
use crate::error::AIProxyError;
use ndarray::{Array, Ix2, Ix3, Ix4};
use tokenizers::Encoding;

pub mod normalize;
pub mod resize;
pub mod imagearray_to_ndarray;
pub mod center_crop;
pub mod rescale;
pub mod preprocessor;
pub mod tokenize;
pub mod postprocessor;
pub mod pooling;

pub const CONV_NEXT_FEATURE_EXTRACTOR_CENTER_CROP_THRESHOLD: u32 = 384;

pub trait Preprocessor: Send + Sync {
    fn process(&self, data: PreprocessorData) -> Result<PreprocessorData, AIProxyError>;
}

pub trait Postprocessor: Send + Sync {
    fn process(&self, data: PostprocessorData) -> Result<PostprocessorData, AIProxyError>;
}

pub enum PreprocessorData {
    ImageArray(Vec<ImageArray>),
    NdArray3C(Array<f32, Ix4>),
    Text(Vec<String>),
    EncodedText(Vec<Encoding>),
}

pub enum PostprocessorData {
    NdArray2(Array<f32, Ix2>),
    NdArray3(Array<f32, Ix3>)
}
