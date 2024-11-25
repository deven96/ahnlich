use image::image_dimensions;
use ndarray::{ArrayView, Ix3};
use std::sync::Mutex;
use rayon::iter::{IntoParallelRefMutIterator, ParallelIterator};
use crate::engine::ai::providers::processors::{Preprocessor, PreprocessorData};
use crate::error::AIProxyError;

pub struct ImageArrayToNdArray;

impl Preprocessor for ImageArrayToNdArray {
    fn process(&self, data: PreprocessorData) -> Result<PreprocessorData, AIProxyError> {
        match data {
            PreprocessorData::ImageArray(mut arrays) => {
                let mut array_shapes = Mutex::new(vec![]);
                let mut array_views = Mutex::new(vec![]);
                arrays.par_iter_mut().for_each(|image_arr| {
                    image_arr.onnx_transform();
                    array_shapes.lock().unwrap().push(image_arr.image_dim());
                    array_views.lock().unwrap().push(image_arr.view());
                });

                let array_shapes = array_shapes.into_inner().unwrap();
                let array_views = array_views.into_inner().unwrap();

                let pixel_values_array = ndarray::stack(ndarray::Axis(0), &array_views)
                    .map_err(|e| AIProxyError::ImageArrayToNdArrayError {
                        message: format!("Images must have same dimensions, instead found: {:?}. \
                        NB: Dimensions listed are not in same order as images provided.", array_shapes),
                    })?;
                Ok(PreprocessorData::NdArray3C(pixel_values_array))
            }
            _ => Err(AIProxyError::ImageArrayToNdArrayError {
                message: "ImageArrayToNdArray failed. Expected ImageArray, got NdArray3C".to_string(),
            }),
        }
    }
}
