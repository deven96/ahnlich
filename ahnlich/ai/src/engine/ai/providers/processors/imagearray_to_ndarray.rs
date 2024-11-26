use crate::engine::ai::providers::processors::{Preprocessor, PreprocessorData};
use crate::error::AIProxyError;
use std::sync::Mutex;

pub struct ImageArrayToNdArray;

impl Preprocessor for ImageArrayToNdArray {
    fn process(&self, data: PreprocessorData) -> Result<PreprocessorData, AIProxyError> {
        match data {
            PreprocessorData::ImageArray(mut arrays) => {
                let array_shapes = Mutex::new(vec![]);
                // Not using par_iter_mut here because it messes up the order of the images
                let array_views = arrays
                    .iter_mut()
                    .map(|image_arr| {
                        image_arr.onnx_transform();
                        array_shapes.lock().unwrap().push(image_arr.image_dim());
                        image_arr.view()
                    })
                    .collect::<Vec<_>>();

                let array_shapes = array_shapes.into_inner().unwrap();
                let pixel_values_array =
                    ndarray::stack(ndarray::Axis(0), &array_views).map_err(|_| {
                        AIProxyError::ImageArrayToNdArrayError {
                            message: format!(
                                "Images must have same dimensions, instead found: {:?}.",
                                array_shapes
                            ),
                        }
                    })?;
                Ok(PreprocessorData::NdArray3C(pixel_values_array))
            }
            _ => Err(AIProxyError::ImageArrayToNdArrayError {
                message: "ImageArrayToNdArray failed. Expected ImageArray, got NdArray3C"
                    .to_string(),
            }),
        }
    }
}
