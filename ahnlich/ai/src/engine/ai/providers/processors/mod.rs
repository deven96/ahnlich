use crate::engine::ai::models::ImageArray;
use crate::error::AIProxyError;
use ndarray::{Array, Ix2, Ix3, Ix4};
use ort::SessionOutputs;
use tokenizers::Encoding;

pub mod center_crop;
pub mod imagearray_to_ndarray;
pub mod normalize;
mod onnx_output_transform;
pub mod pooling;
pub mod postprocessor;
pub mod preprocessor;
pub mod rescale;
pub mod resize;
pub mod tokenize;

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

impl PreprocessorData {
    pub fn into_ndarray3c(self) -> Result<Array<f32, Ix4>, AIProxyError> {
        match self {
            PreprocessorData::NdArray3C(array) => Ok(array),
            _ => Err(AIProxyError::ModelProviderPreprocessingError(
                "`into_ndarray3c` only works for PreprocessorData::NdArray3C".to_string(),
            )),
        }
    }
}

pub enum PostprocessorData<'r, 's> {
    OnnxOutput(SessionOutputs<'r, 's>),
    NdArray2(Array<f32, Ix2>),
    NdArray3(Array<f32, Ix3>),
}
