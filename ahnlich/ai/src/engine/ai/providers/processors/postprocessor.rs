use std::sync::{Arc, Mutex};
use ndarray::{Array, Ix2, Ix3};
use crate::cli::server::SupportedModels;
use crate::engine::ai::providers::processors::normalize::VectorNormalize;
use crate::engine::ai::providers::processors::pooling::{Pooling, RegularPooling, MeanPooling};
use crate::engine::ai::providers::processors::{PostprocessorData, Postprocessor};
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
        let artifacts = match supported_model {
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
            pooling: Arc::new(Mutex::new(artifacts.0)),
            normalize: artifacts.1
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
                message: "Expected NdArray2, got NdArray3".to_string()
            })
        }
    }
}

struct ORTImagePostprocessor {
    model: SupportedModels,
    normalize: VectorNormalize
}