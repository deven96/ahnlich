use std::iter;
use std::sync::{Arc, Mutex};
use hf_hub::api::sync::ApiRepo;
use ndarray::{Array, Ix4};
use tokenizers::{Encoding, Tokenizer};
use crate::engine::ai::models::ImageArray;
use crate::engine::ai::providers::ort_helper::HFConfigReader;
use crate::engine::ai::providers::processors::center_crop::CenterCrop;
use crate::engine::ai::providers::processors::imagearray_to_ndarray::ImageArrayToNdArray;
use crate::engine::ai::providers::processors::normalize::ImageNormalize;
use crate::engine::ai::providers::processors::{Preprocessor, PreprocessorData};
use crate::engine::ai::providers::processors::rescale::Rescale;
use crate::engine::ai::providers::processors::resize::Resize;
use crate::engine::ai::providers::processors::tokenize::Tokenize;
use crate::error::AIProxyError;

#[derive(Clone)]
pub struct ImagePreprocessorFiles {
    pub resize: Option<String>,
    pub normalize: Option<String>,
    pub rescale: Option<String>,
    pub center_crop: Option<String>,
}

impl ImagePreprocessorFiles {
    pub fn iter(&self) -> impl Iterator<Item = (&str, &str)> {
        iter::empty()
            .chain(self.resize.as_ref().map(
                |n| ("resize", n.as_str())))
            .chain(self.normalize.as_ref().map(
                |n| ("normalize", n.as_str())))
            .chain(self.rescale.as_ref().map(
                |n| ("rescale", n.as_str())))
            .chain(self.center_crop.as_ref().map(
                |n| ("center_crop", n.as_str())))
    }
}

impl Default for ImagePreprocessorFiles {
    fn default() -> Self {
        Self {
            normalize: Some("preprocessor_config.json".to_string()),
            resize: Some("preprocessor_config.json".to_string()),
            rescale: Some("preprocessor_config.json".to_string()),
            center_crop: Some("preprocessor_config.json".to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenizerFiles {
    pub tokenizer_file: String,
    pub config_file: String,
    pub special_tokens_map_file: String,
    pub tokenizer_config_file: String,
}

impl Default for TokenizerFiles {
    fn default() -> Self {
        Self {
            tokenizer_file: "tokenizer.json".to_string(),
            config_file: "config.json".to_string(),
            special_tokens_map_file: "special_tokens_map.json".to_string(),
            tokenizer_config_file: "tokenizer_config.json".to_string(),
        }
    }
}

#[derive(Default, Clone)]
pub struct TextPreprocessorFiles {
    pub tokenize: TokenizerFiles,
}

pub enum ORTPreprocessor {
    Image(ORTImagePreprocessor),
    Text(ORTTextPreprocessor),
}

#[derive(Default)]
pub struct ORTImagePreprocessor {
    imagearray_to_ndarray: Option<Box<dyn Preprocessor>>,
    normalize: Option<Box<dyn Preprocessor>>,
    resize: Option<Box<dyn Preprocessor>>,
    rescale: Option<Box<dyn Preprocessor>>,
    center_crop: Option<Box<dyn Preprocessor>>,
}

impl ORTImagePreprocessor {
    pub fn iter(&self) -> impl Iterator<Item = (
        &str, &Box<dyn Preprocessor>)> {
        iter::empty()
            .chain(self.resize.as_ref().map(
                |f| ("resize", f)))
            .chain(self.center_crop.as_ref().map(
                |f| ("center_crop", f)))
            .chain(self.imagearray_to_ndarray.as_ref().map(
                |f| ("imagearray_to_ndarray", f)))
            .chain(self.rescale.as_ref().map(
                |f| ("rescale", f)))
            .chain(self.normalize.as_ref().map(
                |f| ("normalize", f)))
    }

    pub fn load(&mut self, model_repo: ApiRepo, processor_files: ImagePreprocessorFiles) -> Result<(), AIProxyError> {
        let mut type_and_configs: Vec<(&str, Option<serde_json::Value>)> = vec![
            ("imagearray_to_ndarray", None)
        ];

        let mut config_reader = HFConfigReader::new(model_repo);
        for data in processor_files.iter() {
            type_and_configs.push((data.0, Some(config_reader.read(data.1)?)));
        }
        for (processor_type, config) in type_and_configs {
            match processor_type {
                "imagearray_to_ndarray" => {
                    self.imagearray_to_ndarray = Some(Box::new(ImageArrayToNdArray));
                }
                "resize" => {
                    self.resize = Some(Box::new(Resize::try_from(&config.expect("Config exists"))?));
                }
                "normalize" => {
                    self.normalize = Some(Box::new(ImageNormalize::try_from(&config.expect("Config exists"))?));
                }
                "rescale" => {
                    self.rescale = Some(Box::new(Rescale::try_from(&config.expect("Config exists"))?));
                }
                "center_crop" => {
                    self.center_crop = Some(Box::new(CenterCrop::try_from(&config.expect("Config exists"))?));
                }
                _ => return Err(AIProxyError::ModelProviderPreprocessingError(
                    format!("The {} operation not found in ImagePreprocessor.", processor_type)
                ))
            }
        }
        Ok(())
    }

    pub fn process(&self, data: Vec<ImageArray>) -> Result<Array<f32, Ix4>, AIProxyError> {
        let mut data = PreprocessorData::ImageArray(data);
        for (_, processor) in self.iter() {
            data = processor.process(data)?;
        }
        match data {
            PreprocessorData::NdArray3C(array) => Ok(array),
            _ => Err(AIProxyError::ModelProviderPreprocessingError(
                "Expected NdArray after processing".to_string()
            ))
        }
    }
}

pub struct ORTTextPreprocessor {
    pub tokenize: Arc<Mutex<Tokenize>>
}

impl ORTTextPreprocessor {
    pub fn load(model_repo: ApiRepo, processor_files: TextPreprocessorFiles) -> Result<ORTTextPreprocessor, AIProxyError> {
        Ok(
            ORTTextPreprocessor {
                tokenize: Arc::new(Mutex::new(
                    Tokenize::initialize(processor_files.tokenize, model_repo)?,
                )),
            }
        )
    }

    pub fn process(&self, data: Vec<String>, truncate: bool) -> Result<Vec<Encoding>, AIProxyError> {
        let mut data = PreprocessorData::Text(data);
        let mut tokenize = self.tokenize.lock().map_err(|_| {
            AIProxyError::ModelProviderPreprocessingError(
                "Failed to acquire lock on tokenizer".to_string(),
            )
        })?;
        tokenize.set_truncate(truncate);
        data = tokenize.process(data)?;
        match data {
            PreprocessorData::EncodedText(encodings) => Ok(encodings),
            _ => Err(AIProxyError::ModelProviderPreprocessingError(
                "Expected EncodedText after processing".to_string()
            ))
        }
    }
}