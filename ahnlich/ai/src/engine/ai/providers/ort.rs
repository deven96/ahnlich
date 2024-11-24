use crate::cli::server::SupportedModels;
use crate::engine::ai::models::{ImageArray, InputAction, Model, ModelInput};
use crate::engine::ai::providers::processors::tokenize::Tokenize;
use crate::engine::ai::providers::ProviderTrait;
use crate::error::AIProxyError;
use fallible_collections::FallibleVec;
use hf_hub::{api::sync::ApiBuilder, Cache};
use itertools::Itertools;
use rayon::iter::Either;
use ort::{Session, Value};
use rayon::prelude::*;

use ahnlich_types::keyval::StoreKey;
use ndarray::{Array, Array1, Axis, Ix2, Ix3, Ix4, IxDyn, IxDynImpl};
use std::convert::TryFrom;
use std::default::Default;
use std::fmt;
use std::path::{Path, PathBuf};
use std::thread::available_parallelism;
use tokenizers::Encoding;
use crate::engine::ai::providers::processors::preprocessor::{ImagePreprocessorFiles, ORTImagePreprocessor, ORTPreprocessor, ORTTextPreprocessor, TextPreprocessorFiles};
use crate::engine::ai::providers::ort_helper::normalize;
use ndarray::s;
use tokenizers::Tokenizer;
use crate::engine::ai::providers::processors::postprocessor::{ORTPostprocessor, ORTTextPostprocessor};

#[derive(Default)]
pub struct ORTProvider {
    cache_location: Option<PathBuf>,
    cache_location_extension: PathBuf,
    supported_models: Option<SupportedModels>,
    pub preprocessor: Option<ORTPreprocessor>,
    pub postprocessor: Option<ORTPostprocessor>,
    pub model: Option<ORTModel>,
}

impl fmt::Debug for ORTProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ORTProvider")
            .field("cache_location", &self.cache_location)
            .field("cache_location_extension", &self.cache_location_extension)
            .field("supported_models", &self.supported_models)
            .finish()
    }
}

#[derive(Default)]
pub struct ORTImageModel {
    repo_name: String,
    weights_file: String,
    session: Option<Session>,
    input_params: Vec<String>,
    output_param: String,
    preprocessor_files: ImagePreprocessorFiles
}

#[derive(Default)]
pub struct ORTTextModel {
    repo_name: String,
    weights_file: String,
    session: Option<Session>,
    preprocessor_files: TextPreprocessorFiles
}

pub enum ORTModel {
    Image(ORTImageModel),
    Text(ORTTextModel)
}


impl TryFrom<&SupportedModels> for ORTModel {
    type Error = AIProxyError;

    fn try_from(model: &SupportedModels) -> Result<Self, Self::Error> {
        let model_type: Result<ORTModel, AIProxyError> = match model {
            SupportedModels::Resnet50 => Ok(ORTModel::Image(ORTImageModel {
                repo_name: "Qdrant/resnet50-onnx".to_string(),
                weights_file: "model.onnx".to_string(),
                input_params: vec!["input".to_string()],
                output_param: "image_embeds".to_string(),
                ..Default::default()
            })),
            SupportedModels::ClipVitB32Image => Ok(ORTModel::Image(ORTImageModel {
                repo_name: "Qdrant/clip-ViT-B-32-vision".to_string(),
                weights_file: "model.onnx".to_string(),
                input_params: vec!["pixel_values".to_string()],
                output_param: "image_embeds".to_string(),
                ..Default::default()
            })),
            SupportedModels::ClipVitB32Text => Ok(ORTModel::Text(ORTTextModel {
                repo_name: "Qdrant/clip-ViT-B-32-text".to_string(),
                weights_file: "model.onnx".to_string(),
                ..Default::default()
            })),
            SupportedModels::AllMiniLML6V2 => Ok(ORTModel::Text(ORTTextModel {
                repo_name: "Qdrant/all-MiniLM-L6-v2-onnx".to_string(),
                weights_file: "model.onnx".to_string(),
                ..Default::default()
            })),
            SupportedModels::AllMiniLML12V2 => Ok(ORTModel::Text(ORTTextModel {
                repo_name: "Xenova/all-MiniLM-L12-v2".to_string(),
                weights_file: "onnx/model.onnx".to_string(),
                ..Default::default()
            })),
            SupportedModels::BGEBaseEnV15 => Ok(ORTModel::Text(ORTTextModel {
                repo_name: "Xenova/bge-base-en-v1.5".to_string(),
                weights_file: "onnx/model.onnx".to_string(),
                ..Default::default()
            })),
            SupportedModels::BGELargeEnV15 => Ok(ORTModel::Text(ORTTextModel {
                repo_name: "Xenova/bge-large-en-v1.5".to_string(),
                weights_file: "onnx/model.onnx".to_string(),
                ..Default::default()
            })),
            _ => Err(AIProxyError::AIModelNotSupported {
                model_name: model.to_string(),
            }),
        };

        model_type
    }
}

impl ORTProvider {
    pub(crate) fn new() -> Self {
        Self {
            cache_location: None,
            cache_location_extension: PathBuf::from("huggingface"),
            preprocessor: None,
            supported_models: None,
            model: None,
            postprocessor: None,
        }
    }

    fn get_postprocessor() -> Result<(), AIProxyError> {
        Ok(())
    }

    pub fn preprocess_images(&self, data: Vec<ImageArray>) -> Result<Array<f32, Ix4>, AIProxyError> {
        match &self.preprocessor {
            Some(ORTPreprocessor::Image(preprocessor)) => {
                let output_data = preprocessor.process(data)
                    .map_err(
                        |e| AIProxyError::ModelProviderPreprocessingError(
                            format!("Preprocessing failed for {:?} with error: {}",
                                    self.supported_models.unwrap().to_string(), e)
                        ))?;
                Ok(output_data)
            }
            _ => Err(AIProxyError::AIModelNotInitialized)
        }
    }

    pub fn preprocess_texts(&self, data: Vec<String>, truncate: bool) -> Result<Vec<Encoding>, AIProxyError> {
        match &self.preprocessor {
            Some(ORTPreprocessor::Text(preprocessor)) => {
                let output_data = preprocessor.process(data, truncate)
                    .map_err(
                        |e| AIProxyError::ModelProviderPreprocessingError(
                            format!("Preprocessing failed for {:?} with error: {}",
                                    self.supported_models.unwrap().to_string(), e)
                        ))?;
                Ok(output_data)
            }
            _ => Err(AIProxyError::ModelPreprocessingError {
                model_name: self.supported_models.unwrap().to_string(),
                message: "Preprocessor not initialized".to_string(),
            })
        }
    }

    pub fn postprocess_text_embeddings(&self, embeddings: Array<f32, IxDyn>, attention_mask: Array<i64, Ix2>) -> Result<Array<f32, Ix2>, AIProxyError> {
        let embeddings = match embeddings.shape().len() {
            3 => {
                let existing_shape = embeddings.shape().to_vec();
                Ok(embeddings.into_dimensionality()
                    .map_err(
                        |e| AIProxyError::ModelPostprocessingError {
                            model_name: self.supported_models.unwrap().to_string(),
                            message: format!("Unable to convert into 3D array. Existing shape {:?}. {:?}", existing_shape, e.to_string())
                        })?.to_owned())
            }
            2 => {
                let existing_shape = embeddings.shape().to_vec();
                let intermediate = embeddings.into_dimensionality()
                    .map_err(
                        |e| AIProxyError::ModelPostprocessingError {
                            model_name: self.supported_models.unwrap().to_string(),
                            message: format!("Unable to convert into 2D. Existing shape {:?}. {:?}", existing_shape, e.to_string())
                        })?.to_owned();
                return Ok(intermediate)
            }
            _ => {
                Err(AIProxyError::ModelPostprocessingError {
                    model_name: self.supported_models.unwrap().to_string(),
                    message: format!("Unsupported shape for postprocessing. Shape: {:?}", embeddings.shape())
                })
            }
        }?;
        match &self.postprocessor {
            Some(ORTPostprocessor::Text(postprocessor)) => {
                let output_data = postprocessor.process(embeddings, attention_mask)
                    .map_err(
                        |e| AIProxyError::ModelProviderPostprocessingError(
                            format!("Postprocessing failed for {:?} with error: {}",
                                    self.supported_models.unwrap().to_string(), e)
                        ))?;
                Ok(output_data)
            }
            _ => Err(AIProxyError::ModelPostprocessingError {
                model_name: self.supported_models.unwrap().to_string(),
                message: "Postprocessor not initialized".to_string(),
            })
        }
    }

    pub fn batch_inference_image(&self, inputs: Vec<ImageArray>) -> Result<Vec<StoreKey>, AIProxyError> {
        let model = match &self.model {
            Some(ORTModel::Image(model)) => model,
            _ => return Err(AIProxyError::AIModelNotSupported { model_name: self.supported_models.unwrap().to_string() }),
        };
        let pixel_values_array = self.preprocess_images(inputs)?;
        match &model.session {
            Some(session) => {
                let session_inputs = ort::inputs![
                    model.input_params.first().expect("Hardcoded in parameters")
                    .as_str() => pixel_values_array.view(),
                ].map_err(|e| AIProxyError::ModelProviderPreprocessingError(e.to_string()))?;

                let outputs = session.run(session_inputs)
                    .map_err(|e| AIProxyError::ModelProviderRunInferenceError(e.to_string()))?;
                let last_hidden_state_key = match outputs.len() {
                    1 => outputs.keys().next().unwrap(),
                    _ => model.output_param.as_str(),
                };

                let output_data = outputs[last_hidden_state_key]
                    .try_extract_tensor::<f32>()
                    .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;
                let store_keys = output_data
                    .axis_iter(Axis(0))
                    .into_par_iter()
                    .map(|row| {
                        let embeddings = normalize(row.as_slice().unwrap());
                        StoreKey(<Array1<f32>>::from(embeddings))
                    })
                    .collect();
                Ok(store_keys)
            }
            None => Err(AIProxyError::AIModelNotInitialized)
        }
    }

    pub fn batch_inference_text(&self, encodings: Vec<Encoding>) -> Result<Vec<StoreKey>, AIProxyError> {
        let model = match &self.model {
            Some(ORTModel::Text(model)) => model,
            _ => return Err(AIProxyError::AIModelNotSupported { model_name: self.supported_models.unwrap().to_string() }),
        };
        let batch_size = encodings.len();
        // Extract the encoding length and batch size
        let encoding_length = encodings[0].len();
        let max_size = encoding_length * batch_size;

        match &model.session {
            Some(session) => {
                let need_token_type_ids = session
                    .inputs
                    .iter()
                    .any(|input| input.name == "token_type_ids");
                // Preallocate arrays with the maximum size
                let mut ids_array = Vec::with_capacity(max_size);
                let mut mask_array = Vec::with_capacity(max_size);
                let mut token_type_ids_array: Option<Vec<i64>> = None;
                if need_token_type_ids {
                    token_type_ids_array = Some(Vec::with_capacity(max_size));
                }

                // Not using par_iter because the closure needs to be FnMut
                encodings.iter().for_each(|encoding| {
                    let ids = encoding.get_ids();
                    let mask = encoding.get_attention_mask();

                    // Extend the preallocated arrays with the current encoding
                    // Requires the closure to be FnMut
                    ids_array.extend(ids.iter().map(|x| *x as i64));
                    mask_array.extend(mask.iter().map(|x| *x as i64));
                    match token_type_ids_array {
                        Some(ref mut token_type_ids_array) => {
                            token_type_ids_array.extend(encoding.get_type_ids().iter().map(|x| *x as i64));
                        }
                        None => {}
                    }
                });

                // Create CowArrays from vectors
                let inputs_ids_array =
                    Array::from_shape_vec((batch_size, encoding_length), ids_array)
                        .map_err(|e| {
                            AIProxyError::ModelProviderPreprocessingError(e.to_string())
                        })?;

                let attention_mask_array =
                    Array::from_shape_vec((batch_size, encoding_length), mask_array).map_err(|e| {
                        AIProxyError::ModelProviderPreprocessingError(e.to_string())
                    })?;

                let token_type_ids_array = match token_type_ids_array {
                    Some(token_type_ids_array) => {
                        Some(Array::from_shape_vec((batch_size, encoding_length), token_type_ids_array)
                            .map_err(|e| {
                                AIProxyError::ModelProviderPreprocessingError(e.to_string())
                            })?)
                    },
                    None => None,
                };

                let mut session_inputs = ort::inputs![
                    "input_ids" => Value::from_array(inputs_ids_array)?,
                    "attention_mask" => Value::from_array(attention_mask_array.view())?
                ].map_err(|e| AIProxyError::ModelProviderPreprocessingError(e.to_string()))?;
                match token_type_ids_array {
                    Some(token_type_ids_array) => {
                        session_inputs.push((
                            "token_type_ids".into(),
                            Value::from_array(token_type_ids_array)?.into(),
                        ));
                    }
                    None => {}
                }

                let output_key = session.outputs.first().expect("Must exist").name.clone();
                let session_outputs = session.run(session_inputs)
                    .map_err(|e| AIProxyError::ModelProviderRunInferenceError(e.to_string()))?;
                let session_output = session_outputs[output_key.as_str()]
                    .try_extract_tensor::<f32>()
                    .map_err(|e| AIProxyError::ModelProviderPostprocessingError(e.to_string()))?;
                let session_output = session_output
                    .to_owned();
                let embeddings = self.postprocess_text_embeddings(session_output, attention_mask_array)?;
                println!("Embeddings: {:?}", embeddings);
                let store_keys = embeddings
                    .axis_iter(Axis(0))
                    .into_par_iter()
                    .map(|embedding| StoreKey(<Array1<f32>>::from(embedding.to_owned())))
                    .collect();
                Ok(store_keys)
            }
            None => Err(AIProxyError::AIModelNotInitialized),
        }
    }
}


impl ProviderTrait for ORTProvider {
    fn set_cache_location(&mut self, location: &Path) {
        self.cache_location = Some(location.join(self.cache_location_extension.clone()));
    }

    fn set_model(&mut self, model: &SupportedModels) {
        self.supported_models = Some(*model);
    }

    fn load_model(&mut self) -> Result<(), AIProxyError> {
        let Some(cache_location) = self.cache_location.clone() else {
            return Err(AIProxyError::CacheLocationNotInitiailized);
        };
        let Some(supported_model) = self.supported_models else {
            return Err(AIProxyError::AIModelNotInitialized);
        };
        let ort_model = ORTModel::try_from(&supported_model)?;

        let cache = Cache::new(cache_location);
        let api = ApiBuilder::from_cache(cache)
            .with_progress(true)
            .build()
            .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?;

        let threads = available_parallelism()
            .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?
            .get();

        match ort_model {
            ORTModel::Image(ORTImageModel {
                weights_file,
                repo_name,
                                input_params: input_param,
                output_param,
                preprocessor_files,
                ..
            }) => {
                let model_repo = api.model(repo_name.clone());
                let model_file_reference = model_repo
                    .get(&weights_file)
                    .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?;
                let session = Session::builder()?
                    .with_intra_threads(threads)?
                    .commit_from_file(model_file_reference)?;
                self.model = Some(ORTModel::Image(ORTImageModel {
                    repo_name,
                    weights_file,
                    input_params: input_param,
                    output_param,
                    session: Some(session),
                    preprocessor_files: preprocessor_files.clone(),
                    ..Default::default()
                }));
                let mut preprocessor = ORTImagePreprocessor::default();
                preprocessor.load(model_repo, preprocessor_files)?;
                self.preprocessor = Some(ORTPreprocessor::Image(preprocessor)
                );
            },
            ORTModel::Text(ORTTextModel {
                weights_file,
                repo_name,
                preprocessor_files,
                ..
            }) => {
                let model_repo = api.model(repo_name.clone());
                let model_file_reference = model_repo
                    .get(&weights_file)
                    .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?;
                let session = Session::builder()?
                    .with_intra_threads(threads)?
                    .commit_from_file(model_file_reference)?;
                self.model = Some(ORTModel::Text(ORTTextModel {
                    repo_name,
                    weights_file,
                    session: Some(session),
                    preprocessor_files: preprocessor_files.clone(),
                }));
                let preprocessor = ORTTextPreprocessor::load(model_repo, preprocessor_files)?;
                self.preprocessor = Some(ORTPreprocessor::Text(preprocessor));
                let postprocessor = ORTTextPostprocessor::load(supported_model)?;
                self.postprocessor = Some(ORTPostprocessor::Text(postprocessor));
            }
        }
        Ok(())
    }

    fn get_model(&self) -> Result<(), AIProxyError> {
        let Some(cache_location) = self.cache_location.clone() else {
            return Err(AIProxyError::CacheLocationNotInitiailized);
        };
        let supported_model = self
            .supported_models
            .ok_or(AIProxyError::AIModelNotInitialized)?;
        let ort_model = ORTModel::try_from(&supported_model)?;

        let cache = Cache::new(cache_location);
        let api = ApiBuilder::from_cache(cache)
            .with_progress(true)
            .build()
            .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?;

        match ort_model {
            ORTModel::Image(ORTImageModel {
                repo_name,
                weights_file,
                ..
            }) => {
                let model_repo = api.model(repo_name);
                model_repo
                    .get(&weights_file)
                    .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?;
                model_repo
                    .get("preprocessor_config.json")
                    .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?;
                Ok(())
            },
            ORTModel::Text(ORTTextModel {
                repo_name,
                preprocessor_files,
                ..
            }) => {
                let model_repo = api.model(repo_name.clone());
                Tokenize::download_artifacts(preprocessor_files.tokenize, model_repo)?;
                Ok(())
            }
        }
    }

    fn run_inference(
        &self,
        input: ModelInput,
        _action_type: &InputAction,
    ) -> Result<Vec<StoreKey>, AIProxyError> {

        match input {
            ModelInput::Images(images) => self.batch_inference_image(images),
            ModelInput::Texts(encodings) => {
                let mut store_keys: Vec<_> = FallibleVec::try_with_capacity(
                    encodings.len()
                )?;

                for batch_encoding in encodings.into_iter().chunks(16).into_iter() {
                    store_keys.extend(self.batch_inference_text(batch_encoding.collect())?);
                }
                Ok(store_keys)
            },
        }
    }
}
