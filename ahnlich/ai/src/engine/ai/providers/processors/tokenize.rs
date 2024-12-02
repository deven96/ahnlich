use crate::engine::ai::providers::ort_helper::{read_file_to_bytes, HFConfigReader};
use crate::engine::ai::providers::processors::{Preprocessor, PreprocessorData};
use crate::error::AIProxyError;
use hf_hub::api::sync::ApiRepo;
use serde_json::Value;
use tokenizers::decoders::bpe::BPEDecoder;
use tokenizers::{AddedToken, PaddingParams, PaddingStrategy, Tokenizer, TruncationParams};

pub struct Tokenize {
    tokenizer: Tokenizer,
    model_max_length: usize,
    truncate: bool,
}

pub struct TokenizeArtifacts {
    pub tokenizer_bytes: Vec<u8>,
    pub config: Value,
    pub special_tokens_map: Value,
    pub tokenizer_config: Value,
}

impl Tokenize {
    pub fn download_artifacts(
        tokenizer_files: TokenizerFiles,
        model_repo: ApiRepo,
    ) -> Result<TokenizeArtifacts, AIProxyError> {
        let tokenizer_bytes = read_file_to_bytes(
            &model_repo
                .get(&tokenizer_files.tokenizer_file)
                .map_err(|e| AIProxyError::ModelConfigLoadError {
                    message: format!("failed to fetch {}, {}", &tokenizer_files.tokenizer_file, e),
                })?,
        )?;
        let mut config_reader = HFConfigReader::new(model_repo);
        let config = config_reader.read(&tokenizer_files.config_file)?;
        let special_tokens_map = config_reader.read(&tokenizer_files.special_tokens_map_file)?;
        let tokenizer_config = config_reader.read(&tokenizer_files.tokenizer_config_file)?;
        Ok(TokenizeArtifacts {
            tokenizer_bytes,
            config,
            special_tokens_map,
            tokenizer_config,
        })
    }

    pub fn initialize(
        tokenizer_files: TokenizerFiles,
        model_repo: ApiRepo,
    ) -> Result<Self, AIProxyError> {
        let artifacts = Self::download_artifacts(tokenizer_files, model_repo)?;
        let mut tokenizer = Tokenizer::from_bytes(artifacts.tokenizer_bytes).map_err(|_| {
            AIProxyError::ModelTokenizerLoadError {
                message: "Error building Tokenizer from bytes.".to_string(),
            }
        })?;

        //For BGEBaseSmall, the model_max_length value is set to 1000000000000000019884624838656. Which fits in a f64
        let model_max_length = artifacts.tokenizer_config["model_max_length"]
            .as_f64()
            .ok_or(AIProxyError::ModelTokenizerLoadError {
                message: "Error reading model_max_length from tokenizer_config".to_string(),
            })? as usize;
        let pad_id = artifacts.config["pad_token_id"].as_u64().unwrap_or(0) as u32;
        let pad_token = artifacts.tokenizer_config["pad_token"]
            .as_str()
            .ok_or(AIProxyError::ModelTokenizerLoadError {
                message: "Error reading pad_token from tokenizer_config".to_string(),
            })?
            .into();

        let mut tokenizer = tokenizer
            .with_padding(Some(PaddingParams {
                // TODO: the user should able to choose the padding strategy
                strategy: PaddingStrategy::BatchLongest,
                pad_token,
                pad_id,
                ..Default::default()
            }))
            .with_truncation(Some(TruncationParams {
                max_length: model_max_length,
                ..Default::default()
            }))
            .map_err(|_| AIProxyError::ModelTokenizerLoadError {
                message: "Error setting padding and truncation params.".to_string(),
            })?
            .clone();
        if let serde_json::Value::Object(root_object) = artifacts.special_tokens_map {
            for (_, value) in root_object.iter() {
                if value.is_string() {
                    tokenizer.add_special_tokens(&[AddedToken {
                        content: value.as_str().unwrap().into(),
                        special: true,
                        ..Default::default()
                    }]);
                } else if value.is_object() {
                    tokenizer.add_special_tokens(&[AddedToken {
                        content: value["content"].as_str().unwrap().into(),
                        special: true,
                        single_word: value["single_word"].as_bool().unwrap(),
                        lstrip: value["lstrip"].as_bool().unwrap(),
                        rstrip: value["rstrip"].as_bool().unwrap(),
                        normalized: value["normalized"].as_bool().unwrap(),
                    }]);
                }
            }
        }

        let decoder = BPEDecoder::new("</w>".to_string());
        tokenizer.with_decoder(Some(decoder));

        Ok(Self {
            tokenizer: tokenizer.into(),
            model_max_length,
            truncate: true,
        })
    }

    pub fn set_truncate(&mut self, truncate: bool) -> Result<(), AIProxyError> {
        let tokenizer = if truncate {
            self.tokenizer
                .with_truncation(Some(TruncationParams {
                    max_length: self.model_max_length,
                    ..Default::default()
                }))
                .map_err(|_| AIProxyError::ModelTokenizerLoadError {
                    message: "Error setting truncation params.".to_string(),
                })?
        } else {
            self.tokenizer.with_truncation(None).map_err(|_| {
                AIProxyError::ModelTokenizerLoadError {
                    message: "Error removing truncation params.".to_string(),
                }
            })?
        };
        self.truncate = truncate;
        self.tokenizer = tokenizer.clone().into();
        Ok(())
    }
}

impl Preprocessor for Tokenize {
    fn process(&self, data: PreprocessorData) -> Result<PreprocessorData, AIProxyError> {
        match data {
            PreprocessorData::Text(text) => {
                let tokenized = self
                    .tokenizer
                    .encode_batch(text.clone(), true)
                    .map_err(|_| AIProxyError::ModelTokenizationError {
                        message: format!("Tokenize process failed. Texts: {:?}", text),
                    })?;
                Ok(PreprocessorData::EncodedText(tokenized))
            }
            _ => Err(AIProxyError::ModelTokenizationError {
                message: "Tokenize process failed. Expected Text.".to_string(),
            }),
        }
    }
}

pub struct TokenizerFiles {
    pub tokenizer_file: String,
    pub config_file: String,
    pub special_tokens_map_file: String,
    pub tokenizer_config_file: String,
}
