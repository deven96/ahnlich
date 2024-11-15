use crate::error::AIProxyError;
use crate::engine::ai::providers::processors::{Processor, ProcessorData};
use ndarray::Array;
use std::ops::{Div, Sub};

pub struct Normalize {
    mean: Vec<f32>,
    std: Vec<f32>,
    process: bool
}

impl TryFrom<&serde_json::Value> for Normalize {
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

impl Processor for Normalize {
    fn process(&self, data: ProcessorData) -> Result<ProcessorData, AIProxyError> {
        if !self.process {
            return Ok(data);
        }

        match data {
            ProcessorData::NdArray3C(array) => {
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
                        Ok(ProcessorData::NdArray3C(array_normalized))
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