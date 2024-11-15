use ndarray::{ArrayView, Ix3};
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use crate::engine::ai::providers::processors::{Processor, ProcessorData};
use crate::error::AIProxyError;

pub struct ImageArrayToNdArray;

impl Processor for ImageArrayToNdArray {
    fn process(&self, data: ProcessorData) -> Result<ProcessorData, AIProxyError> {
        match data {
            ProcessorData::ImageArray(mut arrays) => {
                let array_views: Vec<ArrayView<f32, Ix3>> = arrays
                    .par_iter_mut()
                    .map(|image_arr| {
                        image_arr.onnx_transform();
                        image_arr.view()
                    })
                    .collect();

                let pixel_values_array = ndarray::stack(ndarray::Axis(0), &array_views)
                    .map_err(|e| AIProxyError::EmbeddingShapeError(e.to_string()))?;
                Ok(ProcessorData::NdArray3C(pixel_values_array))
            }
            _ => Err(AIProxyError::ImageArrayToNdArrayError {
                message: "ImageArrayToNdArray failed. Expected ImageArray, got NdArray3C".to_string(),
            }),
        }
    }
}
