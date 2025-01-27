use crate::cli::server::SupportedModels;
use crate::engine::ai::models::ImageArray;
use crate::engine::ai::providers::ort::helper::HFConfigReader;
use crate::engine::ai::providers::processors::center_crop::CenterCrop;
use crate::engine::ai::providers::processors::imagearray_to_ndarray::ImageArrayToNdArray;
use crate::engine::ai::providers::processors::normalize::ImageNormalize;
use crate::engine::ai::providers::processors::rescale::Rescale;
use crate::engine::ai::providers::processors::resize::Resize;
use crate::engine::ai::providers::processors::tokenize::{Tokenize, TokenizerFiles};
use crate::engine::ai::providers::processors::{Preprocessor, PreprocessorData};
use crate::error::AIProxyError;
use hf_hub::api::sync::ApiRepo;
use ndarray::{Array, Ix4};
use std::sync::{Arc, Mutex};
use tokenizers::Encoding;

pub enum ORTPreprocessor {
    Image(ORTImagePreprocessor),
    Text(ORTTextPreprocessor),
}

pub struct ORTImagePreprocessor {
    model: SupportedModels,
    imagearray_to_ndarray: ImageArrayToNdArray,
    normalize: Option<ImageNormalize>,
    resize: Option<Resize>,
    rescale: Option<Rescale>,
    center_crop: Option<CenterCrop>,
}

impl ORTImagePreprocessor {
    pub fn load(
        supported_model: SupportedModels,
        model_repo: ApiRepo,
    ) -> Result<Self, AIProxyError> {
        let imagearray_to_ndarray = ImageArrayToNdArray;

        let mut config_reader = HFConfigReader::new(model_repo);
        let config = config_reader.read("preprocessor_config.json")?;

        let resize = Resize::initialize(&config)?;
        let center_crop = CenterCrop::initialize(&config)?;
        let rescale = Rescale::initialize(&config)?;
        let normalize = ImageNormalize::initialize(&config)?;

        Ok(Self {
            model: supported_model,
            imagearray_to_ndarray,
            normalize,
            resize,
            rescale,
            center_crop,
        })
    }

    #[tracing::instrument(skip_all)]
    pub fn process(&self, data: Vec<ImageArray>) -> Result<Array<f32, Ix4>, AIProxyError> {
        let mut data = PreprocessorData::ImageArray(data);
        data = match self.resize {
            Some(ref resize) => {
                resize
                    .process(data)
                    .map_err(|e| AIProxyError::ModelPreprocessingError {
                        model_name: self.model.to_string(),
                        message: format!("Failed to process resize: {}", e),
                    })?
            }
            None => data,
        };

        data =
            match self.center_crop {
                Some(ref center_crop) => center_crop.process(data).map_err(|e| {
                    AIProxyError::ModelPreprocessingError {
                        model_name: self.model.to_string(),
                        message: format!("Failed to process center crop: {}", e),
                    }
                })?,
                None => data,
            };

        data = self.imagearray_to_ndarray.process(data).map_err(|e| {
            AIProxyError::ModelPreprocessingError {
                model_name: self.model.to_string(),
                message: format!("Failed to process imagearray to ndarray: {}", e),
            }
        })?;

        data = match self.rescale {
            Some(ref rescale) => {
                rescale
                    .process(data)
                    .map_err(|e| AIProxyError::ModelPreprocessingError {
                        model_name: self.model.to_string(),
                        message: format!("Failed to process rescale: {}", e),
                    })?
            }
            None => data,
        };

        data = match self.normalize {
            Some(ref normalize) => {
                normalize
                    .process(data)
                    .map_err(|e| AIProxyError::ModelPreprocessingError {
                        model_name: self.model.to_string(),
                        message: format!("Failed to process normalize: {}", e),
                    })?
            }
            None => data,
        };

        match data {
            PreprocessorData::NdArray3C(array) => Ok(array),
            _ => Err(AIProxyError::ModelPreprocessingError {
                model_name: self.model.to_string(),
                message: "Expected NdArray3C after processing".to_string(),
            }),
        }
    }
}

pub struct ORTTextPreprocessor {
    model: SupportedModels,
    tokenize: Arc<Mutex<Tokenize>>,
}

impl ORTTextPreprocessor {
    pub fn load(
        supported_models: SupportedModels,
        model_repo: ApiRepo,
    ) -> Result<ORTTextPreprocessor, AIProxyError> {
        let tokenizer_files = TokenizerFiles {
            tokenizer_file: "tokenizer.json".to_string(),
            config_file: "config.json".to_string(),
            special_tokens_map_file: "special_tokens_map.json".to_string(),
            tokenizer_config_file: "tokenizer_config.json".to_string(),
        };

        Ok(ORTTextPreprocessor {
            model: supported_models,
            tokenize: Arc::new(Mutex::new(Tokenize::initialize(
                tokenizer_files,
                model_repo,
            )?)),
        })
    }

    pub fn process(
        &self,
        data: Vec<String>,
        truncate: bool,
    ) -> Result<Vec<Encoding>, AIProxyError> {
        let mut data = PreprocessorData::Text(data);
        let mut tokenize =
            self.tokenize
                .lock()
                .map_err(|_| AIProxyError::ModelPreprocessingError {
                    model_name: self.model.to_string(),
                    message: "Failed to acquire lock on tokenize.".to_string(),
                })?;
        let _ = tokenize.set_truncate(truncate);
        data = tokenize
            .process(data)
            .map_err(|e| AIProxyError::ModelPreprocessingError {
                model_name: self.model.to_string(),
                message: format!("Failed to process tokenize: {}", e),
            })?;

        match data {
            PreprocessorData::EncodedText(encodings) => Ok(encodings),
            _ => Err(AIProxyError::ModelPreprocessingError {
                model_name: self.model.to_string(),
                message: "Expected EncodedText after processing".to_string(),
            }),
        }
    }
}
