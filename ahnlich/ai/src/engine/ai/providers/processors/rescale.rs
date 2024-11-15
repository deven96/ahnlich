use crate::engine::ai::providers::processors::{Processor, ProcessorData};
use crate::error::AIProxyError;

pub struct Rescale {
    scale: f32,
    process: bool
}

impl TryFrom<&serde_json::Value> for Rescale {
    type Error = AIProxyError;

    fn try_from(config: &serde_json::Value) -> Result<Self, AIProxyError> {
        if !config["do_rescale"].as_bool().unwrap_or(true) {
            return Ok(
                Self {
                    scale: 0f32,
                    process: false
                }
            );
        }

        let default_scale = 1.0/255.0;
        let scale = config["rescale_factor"].as_f64().unwrap_or(default_scale) as f32;
        Ok(Self { scale, process: true })
    }
}

impl Processor for Rescale {
    fn process(&self, data: ProcessorData) -> Result<ProcessorData, AIProxyError> {
        if !self.process {
            return Ok(data);
        }

        match data {
            ProcessorData::NdArray3C(array) => {
                let mut array = array;
                array *= self.scale;
                Ok(ProcessorData::NdArray3C(array))
            },
            _ => Err(AIProxyError::RescaleError {
                message: "Rescale process failed. Expected NdArray3C, got ImageArray".to_string(),
            }),
        }
    }
}