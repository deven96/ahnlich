use crate::engine::ai::providers::processors::{Postprocessor, PostprocessorData};
use crate::error::AIProxyError;
use ndarray::{Ix2, Ix3};

pub struct OnnxOutputTransform {
    output_key: &'static str,
}

impl OnnxOutputTransform {
    pub fn new(output_key: &'static str) -> Self {
        Self { output_key }
    }
}

impl Postprocessor for OnnxOutputTransform {
    fn process(&self, data: PostprocessorData) -> Result<PostprocessorData<'_, '_>, AIProxyError> {
        match data {
            PostprocessorData::OnnxOutput(onnx_output) => {
                let output = onnx_output.get(self.output_key).ok_or_else(|| {
                    AIProxyError::OnnxOutputTransformError {
                        message: format!(
                            "Output key '{}' not found in the OnnxOutput.",
                            self.output_key
                        ),
                    }
                })?;
                let output = output.try_extract_tensor::<f32>().map_err(|_| {
                    AIProxyError::OnnxOutputTransformError {
                        message: "Failed to extract tensor from OnnxOutput.".to_string(),
                    }
                })?;
                match output.ndim() {
                    2 => {
                        let output = output.into_dimensionality::<Ix2>().map_err(|_| {
                            AIProxyError::OnnxOutputTransformError {
                                message: "Failed to convert Dyn tensor to 2D array.".to_string(),
                            }
                        })?;
                        Ok(PostprocessorData::NdArray2(output.to_owned()))
                    }
                    3 => {
                        let output = output.into_dimensionality::<Ix3>().map_err(|_| {
                            AIProxyError::OnnxOutputTransformError {
                                message: "Failed to convert Dyn tensor to 3D array.".to_string(),
                            }
                        })?;
                        Ok(PostprocessorData::NdArray3(output.to_owned()))
                    }
                    _ => Err(AIProxyError::OnnxOutputTransformError {
                        message: "Only 2D and 3D tensors are supported.".to_string(),
                    }),
                }
            }
            _ => Err(AIProxyError::OnnxOutputTransformError {
                message: "Only OnnxOutput is supported".to_string(),
            }),
        }
    }
}
