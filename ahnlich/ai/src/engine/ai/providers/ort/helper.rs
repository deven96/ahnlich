use crate::error::AIProxyError;
use hf_hub::api::sync::ApiRepo;
use ndarray::IxDyn;
use ort::tensor::Shape;
use serde_json;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

/// Public function to read a file to bytes.
/// To be used when loading local model files.
pub fn read_file_to_bytes(file: &PathBuf) -> Result<Vec<u8>, AIProxyError> {
    let mut file = File::open(file).map_err(|_| AIProxyError::ModelConfigLoadError {
        message: format!("failed to open file {file:?}"),
    })?;
    let file_size = file
        .metadata()
        .map_err(|_| AIProxyError::ModelConfigLoadError {
            message: format!("failed to get metadata for file {file:?}"),
        })?
        .len() as usize;
    let mut buffer = Vec::with_capacity(file_size);
    file.read_to_end(&mut buffer)
        .map_err(|_| AIProxyError::ModelConfigLoadError {
            message: format!("failed to read file {file:?}"),
        })?;
    Ok(buffer)
}

/// Convert ort tensor tuple (&Shape, &[T]) to ndarray Array (owned)
pub fn tensor_to_ndarray<T: Clone>(
    shape: &Shape,
    data: &[T],
) -> Result<ndarray::Array<T, IxDyn>, AIProxyError> {
    let dims: Vec<usize> = shape.as_ref().iter().map(|&d| d as usize).collect();

    ndarray::Array::from_shape_vec(dims, data.to_vec())
        .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))
}

pub struct HFConfigReader {
    model_repo: ApiRepo,
    cache: HashMap<String, Result<serde_json::Value, AIProxyError>>,
}

impl HFConfigReader {
    pub fn new(model_repo: ApiRepo) -> Self {
        Self {
            model_repo,
            cache: HashMap::new(),
        }
    }

    pub fn read(&mut self, config_name: &str) -> Result<serde_json::Value, AIProxyError> {
        if let Some(value) = self.cache.get(config_name) {
            return value.clone();
        }
        let file =
            self.model_repo
                .get(config_name)
                .map_err(|e| AIProxyError::ModelConfigLoadError {
                    message: format!("failed to fetch {config_name}, {e}"),
                })?;
        let contents =
            read_file_to_bytes(&file).map_err(|e| AIProxyError::ModelConfigLoadError {
                message: format!("failed to read {config_name}, {e}"),
            })?;
        let value: serde_json::Value =
            serde_json::from_slice(&contents).map_err(|e| AIProxyError::ModelConfigLoadError {
                message: format!("failed to parse {config_name}, {e}"),
            })?;
        self.cache
            .insert(config_name.to_string(), Ok(value.clone()));
        Ok(value)
    }
}
