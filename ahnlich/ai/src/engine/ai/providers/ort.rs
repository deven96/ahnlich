use crate::cli::server::SupportedModels;
use crate::engine::ai::models::{ImageArray, InputAction, ModelInput};
use crate::engine::ai::providers::ProviderTrait;
use crate::error::AIProxyError;
use fallible_collections::FallibleVec;
use hf_hub::{api::sync::ApiBuilder, Cache};
use itertools::Itertools;
use ort::{CUDAExecutionProvider, CoreMLExecutionProvider, TensorRTExecutionProvider};
use ort::{Session, SessionOutputs, Value};
use rayon::prelude::*;

use crate::engine::ai::providers::processors::postprocessor::{
    ORTImagePostprocessor, ORTPostprocessor, ORTTextPostprocessor,
};
use crate::engine::ai::providers::processors::preprocessor::{
    ORTImagePreprocessor, ORTPreprocessor, ORTTextPreprocessor,
};
use ahnlich_types::keyval::StoreKey;
use ndarray::{Array, Array1, Axis, Ix2, Ix4};
use std::convert::TryFrom;
use std::default::Default;
use std::fmt;
use std::path::{Path, PathBuf};
use std::thread::available_parallelism;
use tokenizers::Encoding;

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
}

#[derive(Default)]
pub struct ORTTextModel {
    repo_name: String,
    weights_file: String,
    session: Option<Session>,
}

pub enum ORTModel {
    Image(ORTImageModel),
    Text(ORTTextModel),
}

impl TryFrom<&SupportedModels> for ORTModel {
    type Error = AIProxyError;

    fn try_from(model: &SupportedModels) -> Result<Self, Self::Error> {
        let model_type: Result<ORTModel, AIProxyError> = match model {
            SupportedModels::Resnet50 => Ok(ORTModel::Image(ORTImageModel {
                repo_name: "Qdrant/resnet50-onnx".to_string(),
                weights_file: "model.onnx".to_string(),
                ..Default::default()
            })),
            SupportedModels::ClipVitB32Image => Ok(ORTModel::Image(ORTImageModel {
                repo_name: "Qdrant/clip-ViT-B-32-vision".to_string(),
                weights_file: "model.onnx".to_string(),
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

    pub fn preprocess_images(
        &self,
        data: Vec<ImageArray>,
    ) -> Result<Array<f32, Ix4>, AIProxyError> {
        match &self.preprocessor {
            Some(ORTPreprocessor::Image(preprocessor)) => {
                let output_data = preprocessor.process(data).map_err(|e| {
                    AIProxyError::ModelProviderPreprocessingError(format!(
                        "Preprocessing failed for {:?} with error: {}",
                        self.supported_models.unwrap().to_string(),
                        e
                    ))
                })?;
                Ok(output_data)
            }
            _ => Err(AIProxyError::AIModelNotInitialized),
        }
    }

    pub fn preprocess_texts(
        &self,
        data: Vec<String>,
        truncate: bool,
    ) -> Result<Vec<Encoding>, AIProxyError> {
        match &self.preprocessor {
            Some(ORTPreprocessor::Text(preprocessor)) => {
                let output_data = preprocessor.process(data, truncate).map_err(|e| {
                    AIProxyError::ModelProviderPreprocessingError(format!(
                        "Preprocessing failed for {:?} with error: {}",
                        self.supported_models.unwrap().to_string(),
                        e
                    ))
                })?;
                Ok(output_data)
            }
            _ => Err(AIProxyError::ModelPreprocessingError {
                model_name: self.supported_models.unwrap().to_string(),
                message: "Preprocessor not initialized".to_string(),
            }),
        }
    }

    pub fn postprocess_text_output(
        &self,
        session_output: SessionOutputs,
        attention_mask: Array<i64, Ix2>,
    ) -> Result<Array<f32, Ix2>, AIProxyError> {
        match &self.postprocessor {
            Some(ORTPostprocessor::Text(postprocessor)) => {
                let output_data = postprocessor
                    .process(session_output, attention_mask)
                    .map_err(|e| {
                        AIProxyError::ModelProviderPostprocessingError(format!(
                            "Postprocessing failed for {:?} with error: {}",
                            self.supported_models.unwrap().to_string(),
                            e
                        ))
                    })?;
                Ok(output_data)
            }
            _ => Err(AIProxyError::ModelPostprocessingError {
                model_name: self.supported_models.unwrap().to_string(),
                message: "Postprocessor not initialized".to_string(),
            }),
        }
    }

    pub fn postprocess_image_output(
        &self,
        session_output: SessionOutputs,
    ) -> Result<Array<f32, Ix2>, AIProxyError> {
        match &self.postprocessor {
            Some(ORTPostprocessor::Image(postprocessor)) => {
                let output_data = postprocessor.process(session_output).map_err(|e| {
                    AIProxyError::ModelProviderPostprocessingError(format!(
                        "Postprocessing failed for {:?} with error: {}",
                        self.supported_models.unwrap().to_string(),
                        e
                    ))
                })?;
                Ok(output_data)
            }
            _ => Err(AIProxyError::ModelPostprocessingError {
                model_name: self.supported_models.unwrap().to_string(),
                message: "Postprocessor not initialized".to_string(),
            }),
        }
    }

    pub fn batch_inference_image(
        &self,
        inputs: Array<f32, Ix4>,
    ) -> Result<Array<f32, Ix2>, AIProxyError> {
        let model = match &self.model {
            Some(ORTModel::Image(model)) => model,
            _ => {
                return Err(AIProxyError::AIModelNotSupported {
                    model_name: self.supported_models.unwrap().to_string(),
                })
            }
        };
        match &model.session {
            Some(session) => {
                let input_param = match self.supported_models.unwrap() {
                    SupportedModels::Resnet50 => "input",
                    SupportedModels::ClipVitB32Image => "pixel_values",
                    _ => {
                        return Err(AIProxyError::AIModelNotSupported {
                            model_name: self.supported_models.unwrap().to_string(),
                        })
                    }
                };

                let session_inputs = ort::inputs![
                    input_param => inputs.view(),
                ]
                .map_err(|e| AIProxyError::ModelProviderPreprocessingError(e.to_string()))?;

                let outputs = session
                    .run(session_inputs)
                    .map_err(|e| AIProxyError::ModelProviderRunInferenceError(e.to_string()))?;
                let embeddings = self.postprocess_image_output(outputs)?;
                Ok(embeddings)
            }
            None => Err(AIProxyError::AIModelNotInitialized),
        }
    }

    pub fn batch_inference_text(
        &self,
        encodings: Vec<Encoding>,
    ) -> Result<Array<f32, Ix2>, AIProxyError> {
        let model = match &self.model {
            Some(ORTModel::Text(model)) => model,
            _ => {
                return Err(AIProxyError::AIModelNotSupported {
                    model_name: self.supported_models.unwrap().to_string(),
                })
            }
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
                    if let Some(ref mut token_type_ids_array) = token_type_ids_array {
                        token_type_ids_array
                            .extend(encoding.get_type_ids().iter().map(|x| *x as i64));
                    }
                });

                // Create CowArrays from vectors
                let inputs_ids_array =
                    Array::from_shape_vec((batch_size, encoding_length), ids_array).map_err(
                        |e| AIProxyError::ModelProviderPreprocessingError(e.to_string()),
                    )?;

                let attention_mask_array =
                    Array::from_shape_vec((batch_size, encoding_length), mask_array).map_err(
                        |e| AIProxyError::ModelProviderPreprocessingError(e.to_string()),
                    )?;

                let token_type_ids_array = match token_type_ids_array {
                    Some(token_type_ids_array) => Some(
                        Array::from_shape_vec((batch_size, encoding_length), token_type_ids_array)
                            .map_err(|e| {
                                AIProxyError::ModelProviderPreprocessingError(e.to_string())
                            })?,
                    ),
                    None => None,
                };

                let mut session_inputs = ort::inputs![
                    "input_ids" => Value::from_array(inputs_ids_array)?,
                    "attention_mask" => Value::from_array(attention_mask_array.view())?
                ]
                .map_err(|e| AIProxyError::ModelProviderPreprocessingError(e.to_string()))?;

                if let Some(token_type_ids_array) = token_type_ids_array {
                    session_inputs.push((
                        "token_type_ids".into(),
                        Value::from_array(token_type_ids_array)?.into(),
                    ));
                }

                let session_outputs = session
                    .run(session_inputs)
                    .map_err(|e| AIProxyError::ModelProviderRunInferenceError(e.to_string()))?;
                let embeddings =
                    self.postprocess_text_output(session_outputs, attention_mask_array)?;
                Ok(embeddings.to_owned())
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
        ort::init()
            .with_execution_providers([
                // Prefer TensorRT over CUDA.
                TensorRTExecutionProvider::default().build(),
                CUDAExecutionProvider::default().build(),
                // Or use ANE on Apple platforms
                CoreMLExecutionProvider::default().build(),
            ])
            .commit()?;

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
                    session: Some(session),
                }));
                let preprocessor =
                    ORTImagePreprocessor::load(self.supported_models.unwrap(), model_repo)?;
                self.preprocessor = Some(ORTPreprocessor::Image(preprocessor));
                let postprocessor = ORTImagePostprocessor::load(supported_model)?;
                self.postprocessor = Some(ORTPostprocessor::Image(postprocessor));
            }
            ORTModel::Text(ORTTextModel {
                weights_file,
                repo_name,
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
                }));
                let preprocessor =
                    ORTTextPreprocessor::load(self.supported_models.unwrap(), model_repo)?;
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

        let (repo_name, weights_file) = match ort_model {
            ORTModel::Image(ORTImageModel {
                repo_name,
                weights_file,
                ..
            }) => (repo_name, weights_file),
            ORTModel::Text(ORTTextModel {
                repo_name,
                weights_file,
                ..
            }) => (repo_name, weights_file),
        };
        let model_repo = api.model(repo_name);
        model_repo
            .get(&weights_file)
            .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?;
        Ok(())
    }

    fn run_inference(
        &self,
        input: ModelInput,
        _action_type: &InputAction,
    ) -> Result<Vec<StoreKey>, AIProxyError> {
        match input {
            ModelInput::Images(images) => {
                let mut store_keys: Vec<StoreKey> = FallibleVec::try_with_capacity(images.len())?;

                for batch_image in images.axis_chunks_iter(Axis(0), 16) {
                    let embeddings = self.batch_inference_image(batch_image.to_owned())?;
                    let new_store_keys: Vec<StoreKey> = embeddings
                        .axis_iter(Axis(0))
                        .into_par_iter()
                        .map(|embedding| StoreKey(<Array1<f32>>::from(embedding.to_owned())))
                        .collect();
                    store_keys.extend(new_store_keys);
                }
                Ok(store_keys)
            }
            ModelInput::Texts(encodings) => {
                let mut store_keys: Vec<StoreKey> =
                    FallibleVec::try_with_capacity(encodings.len())?;

                for batch_encoding in encodings.into_iter().chunks(16).into_iter() {
                    let embeddings = self.batch_inference_text(batch_encoding.collect())?;
                    let new_store_keys: Vec<StoreKey> = embeddings
                        .axis_iter(Axis(0))
                        .into_par_iter()
                        .map(|embedding| StoreKey(<Array1<f32>>::from(embedding.to_owned())))
                        .collect();
                    store_keys.extend(new_store_keys);
                }
                Ok(store_keys)
            }
        }
    }
}
