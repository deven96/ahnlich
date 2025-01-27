use crate::engine::ai::models::OnnxTransformResult;
use crate::engine::ai::providers::processors::{Preprocessor, PreprocessorData};
use crate::error::AIProxyError;

pub struct ImageArrayToNdArray;

impl Preprocessor for ImageArrayToNdArray {
    #[tracing::instrument(skip_all)]
    fn process(&self, data: PreprocessorData) -> Result<PreprocessorData, AIProxyError> {
        match data {
            PreprocessorData::ImageArray(arrays) => {
                // Not using par_iter_mut here because it messes up the order of the images
                let arrays = arrays
                    .into_iter()
                    .map(OnnxTransformResult::try_from)
                    .collect::<Result<Vec<_>, _>>()?;
                let array_views: Vec<_> = arrays.iter().map(|a| a.view()).collect();

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
