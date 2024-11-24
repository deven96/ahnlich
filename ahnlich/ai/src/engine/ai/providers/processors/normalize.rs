use crate::error::AIProxyError;
use crate::engine::ai::providers::processors::{Postprocessor, PostprocessorData, Preprocessor, PreprocessorData};
use ndarray::{Array, Axis};
use std::ops::{Div, Sub};

pub struct ImageNormalize {
    mean: Vec<f32>,
    std: Vec<f32>,
    process: bool
}

impl TryFrom<&serde_json::Value> for ImageNormalize {
    type Error = AIProxyError;

    fn try_from(config: &serde_json::Value) -> Result<Self, AIProxyError> {
        if !config["do_normalize"].as_bool().unwrap_or(false) {
            return Ok(
                Self {
                    mean: vec![],
                    std: vec![],
                    process: false
                }
            );
        }

        fn get_array(value: &serde_json::Value, key: &str) -> Result<Vec<f32>, AIProxyError> {
            let field = value.get(key)
                .ok_or_else(|| AIProxyError::ModelConfigLoadError {
                    message: format!("The key '{}' is missing from the configuration.", key),
                })?;
            serde_json::from_value(field.to_owned()).map_err(|_| AIProxyError::ModelConfigLoadError {
                message: format!("The key '{}' in the configuration must be an array of floats.", key),
            })
        }

        let mean = get_array(config, "image_mean")?;
        let std = get_array(config, "image_std")?;
        Ok(Self { mean, std, process: true })
    }
}

impl Preprocessor for ImageNormalize {
    fn process(&self, data: PreprocessorData) -> Result<PreprocessorData, AIProxyError> {
        if !self.process {
            return Ok(data);
        }

        match data {
            PreprocessorData::NdArray3C(array) => {
                let mean = Array::from_vec(self.mean.clone())
                    .into_shape_with_order((3, 1, 1))
                    .unwrap();
                let std = Array::from_vec(self.std.clone())
                    .into_shape_with_order((3, 1, 1))
                    .unwrap();

                let shape = array.shape().to_vec();
                match shape.as_slice() {
                    [b, c, h, w] => {
                        let mean_broadcast = mean.broadcast((*b, *c, *h, *w)).expect("Broadcast will always succeed.");
                        let std_broadcast = std.broadcast((*b, *c, *h, *w)).expect("Broadcast will always succeed.");
                        let array_normalized = array
                            .sub(mean_broadcast)
                            .div(std_broadcast);
                        Ok(PreprocessorData::NdArray3C(array_normalized))
                    }
                    _ => Err(AIProxyError::ImageNormalizationError {
                        message: format!("Image normalization failed due to invalid shape for image array; \
                expected 4 dimensions, got {} dimensions.", shape.len()),
                    }),
                }
            }
            _ => Err(AIProxyError::ImageNormalizationError {
                message: "Expected NdArray3C, got ImageArray".to_string(),
            }),
        }
    }
}

pub struct VectorNormalize;

impl Postprocessor for VectorNormalize {
    fn process(&self, data: PostprocessorData) -> Result<PostprocessorData, AIProxyError> {
        match data {
            PostprocessorData::NdArray2(array) => {
                let norm = (&array * &array).sum_axis(Axis(1)).sqrt();
                let epsilon = 1e-12;
                let regularized_norm = norm + epsilon;
                let regularized_norm = regularized_norm.insert_axis(Axis(1));
                let source_shape = regularized_norm.shape();
                let target_shape = array.shape();
                let broadcasted_norm = regularized_norm
                    .broadcast(array.dim()).ok_or(AIProxyError::VectorNormalizationError {
                    message: format!("Could not broadcast attention mask with shape {:?} to \
                         shape {:?} of the input tensor.", source_shape, target_shape),
                })?.to_owned();
                Ok(PostprocessorData::NdArray2(array / broadcasted_norm))
            }
            _ => Err(AIProxyError::VectorNormalizationError {
                message: "Expected NdArray2, got NdArray3".to_string(),
            }),
        }
    }
}