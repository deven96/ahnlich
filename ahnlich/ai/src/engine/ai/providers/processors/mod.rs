use crate::engine::ai::models::ImageArray;
use crate::error::AIProxyError;
use ndarray::{Array, Ix4};

pub mod normalize;
pub mod resize;
pub mod imagearray_to_ndarray;
pub mod center_crop;
pub mod rescale;
pub mod preprocessor;

pub const CONV_NEXT_FEATURE_EXTRACTOR_CENTER_CROP_THRESHOLD: u32 = 384;

pub trait Processor: Send + Sync {
    fn process(&self, data: ProcessorData) -> Result<ProcessorData, AIProxyError>;
}

pub enum ProcessorData {
    ImageArray(Vec<ImageArray>),
    NdArray3C(Array<f32, Ix4>)
}
