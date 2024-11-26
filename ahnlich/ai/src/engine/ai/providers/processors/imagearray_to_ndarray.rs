use crate::engine::ai::providers::processors::{Preprocessor, PreprocessorData};
use crate::error::AIProxyError;

pub struct ImageArrayToNdArray;

impl Preprocessor for ImageArrayToNdArray {
    fn process(&self, data: PreprocessorData) -> Result<PreprocessorData, AIProxyError> {
        match data {
            PreprocessorData::ImageArray(mut arrays) => {
                // Not using par_iter_mut here because it messes up the order of the images
                // TODO: Figure out if it's more expensive to use par_iter_mut with enumerate or
                // just keep doing it sequentially
                let array_views = arrays
                    .iter_mut()
                    .map(|image_arr| {
                        image_arr.onnx_transform();
                        image_arr.view()
                    })
                    .collect::<Vec<_>>();

                let pixel_values_array =
                    ndarray::stack(ndarray::Axis(0), &array_views).map_err(|_| {
                        let array_shapes = arrays
                            .iter()
                            .map(|image| image.image_dim())
                            .collect::<Vec<_>>();
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
