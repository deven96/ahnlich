use crate::cli::server::SupportedModels;
use crate::engine::ai::providers::processors::normalize::VectorNormalize;
use crate::engine::ai::providers::processors::onnx_output_transform::OnnxOutputTransform;
use crate::engine::ai::providers::processors::pooling::{MeanPooling, Pooling, RegularPooling};
use crate::engine::ai::providers::processors::{Postprocessor, PostprocessorData};
use crate::error::AIProxyError;
use ndarray::{Array, Ix2};
use ort::SessionOutputs;

use super::pooling::MeanPoolingBuilder;

pub enum ORTPostprocessor {
    Image(ORTImagePostprocessor),
    Text(ORTTextPostprocessor),
}

pub struct ORTTextPostprocessor {
    model: SupportedModels,
    onnx_output_transform: OnnxOutputTransform,
    pooling: Pooling,
    normalize: Option<VectorNormalize>,
}

impl ORTTextPostprocessor {
    pub fn load(supported_model: SupportedModels) -> Result<Self, AIProxyError> {
        let output_transform = match supported_model {
            SupportedModels::AllMiniLML6V2
            | SupportedModels::AllMiniLML12V2
            | SupportedModels::BGEBaseEnV15
            | SupportedModels::BGELargeEnV15 => OnnxOutputTransform::new("last_hidden_state"),
            SupportedModels::ClipVitB32Text => OnnxOutputTransform::new("text_embeds"),
            _ => Err(AIProxyError::ModelPostprocessingError {
                model_name: supported_model.to_string(),
                message: "Unsupported model for ORTTextPostprocessor".to_string(),
            })?,
        };
        let (pooling, normalize) = match supported_model {
            SupportedModels::AllMiniLML6V2 | SupportedModels::AllMiniLML12V2 => {
                (Pooling::Mean(MeanPoolingBuilder), Some(VectorNormalize))
            }
            SupportedModels::BGEBaseEnV15 | SupportedModels::BGELargeEnV15 => {
                (Pooling::Regular(RegularPooling), Some(VectorNormalize))
            }
            SupportedModels::ClipVitB32Text => (Pooling::Mean(MeanPoolingBuilder), None),
            _ => {
                return Err(AIProxyError::ModelPostprocessingError {
                    model_name: supported_model.to_string(),
                    message: "Unsupported model for ORTTextPostprocessor".to_string(),
                });
            }
        };
        Ok(Self {
            model: supported_model,
            onnx_output_transform: output_transform,
            pooling,
            normalize,
        })
    }

    pub fn process(
        &self,
        session_outputs: SessionOutputs,
        attention_mask: Array<i64, Ix2>,
    ) -> Result<Array<f32, Ix2>, AIProxyError> {
        let embeddings = self
            .onnx_output_transform
            .process(PostprocessorData::OnnxOutput(session_outputs))?;
        let pooling_impl = match &self.pooling {
            Pooling::Mean(pooling) => {
                PoolingImpl::Mean(pooling.with_attention_mask(attention_mask))
            }
            Pooling::Regular(a) => PoolingImpl::Regular(*a),
        };
        let pooled = match pooling_impl {
            PoolingImpl::Regular(ref pooling) => pooling.process(embeddings)?,
            PoolingImpl::Mean(ref pooling) => pooling.process(embeddings)?,
        };
        let result = match &self.normalize {
            Some(normalize) => normalize.process(pooled),
            None => Ok(pooled),
        }?;
        match result {
            PostprocessorData::NdArray2(array) => Ok(array),
            _ => Err(AIProxyError::ModelPostprocessingError {
                model_name: self.model.to_string(),
                message: "Only returns NdArray2".to_string(),
            }),
        }
    }
}

enum PoolingImpl {
    Regular(RegularPooling),
    Mean(MeanPooling),
}

pub struct ORTImagePostprocessor {
    model: SupportedModels,
    onnx_output_transform: OnnxOutputTransform,
    normalize: Option<VectorNormalize>,
}

impl ORTImagePostprocessor {
    pub fn load(supported_model: SupportedModels) -> Result<Self, AIProxyError> {
        let output_transform = match supported_model {
            SupportedModels::Resnet50 => OnnxOutputTransform::new("output"),
            SupportedModels::ClipVitB32Image => OnnxOutputTransform::new("image_embeds"),
            _ => Err(AIProxyError::ModelPostprocessingError {
                model_name: supported_model.to_string(),
                message: "Unsupported model for ORTImagePostprocessor".to_string(),
            })?,
        };
        let normalize = match supported_model {
            SupportedModels::Resnet50 => Ok(Some(VectorNormalize)),
            SupportedModels::ClipVitB32Image => Ok(None),
            _ => Err(AIProxyError::ModelPostprocessingError {
                model_name: supported_model.to_string(),
                message: "Unsupported model for ORTImagePostprocessor".to_string(),
            }),
        }?;
        Ok(Self {
            model: supported_model,
            normalize,
            onnx_output_transform: output_transform,
        })
    }

    pub fn process(
        &self,
        session_outputs: SessionOutputs,
    ) -> Result<Array<f32, Ix2>, AIProxyError> {
        let embeddings = self
            .onnx_output_transform
            .process(PostprocessorData::OnnxOutput(session_outputs))?;
        let result = match &self.normalize {
            Some(normalize) => normalize.process(embeddings),
            None => Ok(embeddings),
        }?;
        match result {
            PostprocessorData::NdArray2(array) => Ok(array),
            _ => Err(AIProxyError::ModelPostprocessingError {
                model_name: self.model.to_string(),
                message: "Only returns NdArray2".to_string(),
            }),
        }
    }
}
