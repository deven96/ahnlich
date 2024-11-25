use std::sync::{Arc, Mutex};
use ndarray::{Array, Ix2, Ix3};
use ort::SessionOutputs;
use crate::cli::server::SupportedModels;
use crate::engine::ai::providers::processors::normalize::VectorNormalize;
use crate::engine::ai::providers::processors::pooling::{Pooling, RegularPooling, MeanPooling};
use crate::engine::ai::providers::processors::{PostprocessorData, Postprocessor};
use crate::engine::ai::providers::processors::onnx_output_transform::OnnxOutputTransform;
use crate::error::AIProxyError;

pub enum ORTPostprocessor {
    Image(ORTImagePostprocessor),
    Text(ORTTextPostprocessor),
}

pub struct ORTTextPostprocessor {
    model: SupportedModels,
    pooling: Arc<Mutex<Pooling>>,
    normalize: Option<VectorNormalize>
}


impl ORTTextPostprocessor {
    pub fn load(supported_model: SupportedModels) -> Result<Self, AIProxyError> {
        let ops = match supported_model {
            SupportedModels::AllMiniLML6V2 |
            SupportedModels::AllMiniLML12V2 => Ok((
                Pooling::Mean(MeanPooling::new()),
                Some(VectorNormalize)
            )),
            SupportedModels::BGEBaseEnV15 |
            SupportedModels::BGELargeEnV15 => Ok((
                Pooling::Regular(RegularPooling),
                Some(VectorNormalize)
            )),
            SupportedModels::ClipVitB32Text => Ok((
                Pooling::Mean(MeanPooling::new()),
                None
            )),
            _ => Err(AIProxyError::ModelPostprocessingError {
                model_name: supported_model.to_string(),
                message: "Unsupported model for ORTTextPostprocessor".to_string(),
            })
        }?;
        Ok(Self {
            model: supported_model,
            pooling: Arc::new(Mutex::new(ops.0)),
            normalize: ops.1
        })
    }

    pub fn process(&self, embeddings: Array<f32, Ix3>, attention_mask: Array<i64, Ix2>) -> Result<Array<f32, Ix2>, AIProxyError> {
        let mut pooling = self.pooling.lock().map_err(|_| AIProxyError::ModelPostprocessingError {
            model_name: self.model.to_string(),
            message: "Failed to acquire lock on pooling.".to_string()
        })?;
        let pooled = match &mut *pooling {
            Pooling::Regular(pooling) => {
                pooling.process(PostprocessorData::NdArray3(embeddings))?
            },
            Pooling::Mean(pooling) => {
                pooling.set_attention_mask(Some(attention_mask));
                pooling.process(PostprocessorData::NdArray3(embeddings))?
            }
        };
        let result = match &self.normalize {
            Some(normalize) => normalize.process(pooled),
            None => Ok(pooled)
        }?;
        match result {
            PostprocessorData::NdArray2(array) => Ok(array),
            _ => Err(AIProxyError::ModelPostprocessingError {
                model_name: self.model.to_string(),
                message: "Only returns NdArray2".to_string()
            })
        }
    }
}

pub struct ORTImagePostprocessor {
    model: SupportedModels,
    onnx_output_transform: OnnxOutputTransform,
    normalize: Option<VectorNormalize>
}

impl ORTImagePostprocessor {
    pub fn load(supported_model: SupportedModels) -> Result<Self, AIProxyError> {
        let output_transform = match supported_model {
            SupportedModels::Resnet50 |
            SupportedModels::ClipVitB32Image =>
                OnnxOutputTransform::new("image_embeds".to_string()),
            _ => Err(AIProxyError::ModelPostprocessingError {
                model_name: supported_model.to_string(),
                message: "Unsupported model for ORTImagePostprocessor".to_string()
            })?
        };
        let normalize = match supported_model {
            SupportedModels::Resnet50 => Ok(Some(VectorNormalize)),
            SupportedModels::ClipVitB32Image => Ok(None),
            _ => Err(AIProxyError::ModelPostprocessingError {
                model_name: supported_model.to_string(),
                message: "Unsupported model for ORTImagePostprocessor".to_string()
            })
        }?;
        Ok(Self {
            model: supported_model,
            normalize,
            onnx_output_transform: output_transform
        })
    }

    pub fn process(&self, session_outputs: SessionOutputs) -> Result<Array<f32, Ix2>, AIProxyError> {
        let embeddings = self.onnx_output_transform.process(PostprocessorData::OnnxOutput(session_outputs))?;
        let result = match &self.normalize {
            Some(normalize) => normalize.process(embeddings),
            None => Ok(embeddings)
        }?;
        match result {
            PostprocessorData::NdArray2(array) => Ok(array),
            _ => Err(AIProxyError::ModelPostprocessingError {
                model_name: self.model.to_string(),
                message: "Only returns NdArray2".to_string()
            })
        }
    }
}