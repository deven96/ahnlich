use crate::engine::ai::providers::processors::{Postprocessor, PostprocessorData};
use crate::error::AIProxyError;
use ndarray::{s, Array, Axis, Ix2};

pub enum Pooling {
    Regular(RegularPooling),
    Mean(MeanPooling),
}

pub struct RegularPooling;

impl Postprocessor for RegularPooling {
    fn process(&self, data: PostprocessorData) -> Result<PostprocessorData, AIProxyError> {
        match data {
            PostprocessorData::NdArray3(array) => {
                let processed = array.slice(s![.., 0, ..]).to_owned();
                Ok(PostprocessorData::NdArray2(processed))
            }
            PostprocessorData::NdArray2(array) => Ok(PostprocessorData::NdArray2(array)),
            _ => Err(AIProxyError::PoolingError {
                message: "Expected NdArray3, NdArray2".to_string(),
            }),
        }
    }
}

#[derive(Default)]
pub struct MeanPooling {
    attention_mask: Option<Array<i64, Ix2>>,
}

impl MeanPooling {
    pub fn new() -> Self {
        Self {
            attention_mask: None,
        }
    }

    pub fn set_attention_mask(&mut self, attention_mask: Option<Array<i64, Ix2>>) {
        self.attention_mask = attention_mask;
    }
}

impl Postprocessor for MeanPooling {
    fn process(&self, data: PostprocessorData) -> Result<PostprocessorData, AIProxyError> {
        match data {
            PostprocessorData::NdArray3(array) => {
                let attention_mask = match &self.attention_mask {
                    Some(mask) => {
                        let attention_mask = mask.mapv(|x| x as f32);
                        attention_mask
                            .insert_axis(Axis(2))
                            .broadcast(array.dim())
                            .ok_or(AIProxyError::PoolingError {
                                message: format!(
                                    "Could not broadcast attention mask with shape {:?} to \
                         shape {:?} of the input tensor.",
                                    mask.shape(),
                                    array.shape()
                                ),
                            })?
                            .to_owned()
                    }
                    None => Array::ones(array.dim()),
                };

                let masked_array = &attention_mask * &array;
                let masked_array_sum = masked_array.sum_axis(Axis(1));
                let attention_mask_sum = attention_mask.sum_axis(Axis(1));
                let min_value = 1e-9;
                let attention_mask_sum = attention_mask_sum.mapv(|x| x.max(min_value));
                Ok(PostprocessorData::NdArray2(
                    &masked_array_sum / &attention_mask_sum,
                ))
            }
            PostprocessorData::NdArray2(array) => Ok(PostprocessorData::NdArray2(array)),
            _ => Err(AIProxyError::PoolingError {
                message: "Expected NdArray3, NdArray2".to_string(),
            }),
        }
    }
}
