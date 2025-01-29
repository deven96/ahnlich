use crate::engine::ai::providers::processors::{Preprocessor, PreprocessorData};
use crate::error::AIProxyError;

pub struct Rescale {
    scale: f32,
}

impl Rescale {
    pub fn initialize(config: &serde_json::Value) -> Result<Option<Self>, AIProxyError> {
        if !config["do_rescale"].as_bool().unwrap_or(true) {
            return Ok(None);
        }

        let default_scale = 1.0 / 255.0;
        let scale = config["rescale_factor"].as_f64().unwrap_or(default_scale) as f32;
        Ok(Some(Self { scale }))
    }
}

impl Preprocessor for Rescale {
    #[tracing::instrument(skip_all)]
    fn process(&self, data: PreprocessorData) -> Result<PreprocessorData, AIProxyError> {
        match data {
            PreprocessorData::NdArray3C(array) => {
                let mut array = array;
                array *= self.scale;
                Ok(PreprocessorData::NdArray3C(array))
            }
            _ => Err(AIProxyError::RescaleError {
                message: "Rescale process failed. Expected NdArray3C.".to_string(),
            }),
        }
    }
}
