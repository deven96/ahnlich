use crate::cli::server::SupportedModels;
use crate::engine::ai::models::{InputAction, ImageArray, Model, ModelInput};
use crate::engine::ai::providers::ProviderTrait;
use crate::error::AIProxyError;
use ahnlich_types::ai::AIStoreInputType;
use hf_hub::{api::sync::ApiBuilder, Cache};
use log;
use ort::{ExecutionProviderDispatch, GraphOptimizationLevel, Session};
use rayon::prelude::*;
use rayon::iter::Either;

use std::convert::TryFrom;
use std::default::Default;
use std::fmt;
use std::path::{Path, PathBuf};
use std::thread::available_parallelism;
use ndarray::{Array, ArrayView, Ix3};

#[derive(Default)]
pub struct ORTProvider {
    cache_location: Option<PathBuf>,
    cache_location_extension: PathBuf,
    supported_models: Option<SupportedModels>,
    model: Option<ORTModel>,
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
    input_param: String,
    output_param: String,
}

pub enum ORTModel {
    Image(ORTImageModel),
}

impl TryFrom<&SupportedModels> for ORTModel {
    type Error = AIProxyError;

    fn try_from(model: &SupportedModels) -> Result<Self, Self::Error> {
        let model_type = match model {
            SupportedModels::Resnet50 => Ok(ORTImageModel {
                repo_name: "Qdrant/resnet50-onnx".to_string(),
                weights_file: "model.onnx".to_string(),
                input_param: "input".to_string(),
                output_param: "image_embeds".to_string(),
                ..Default::default()
            }),
            SupportedModels::ClipVitB32 => Ok(ORTImageModel {
                repo_name: "Qdrant/clip-ViT-B-32-vision".to_string(),
                weights_file: "model.onnx".to_string(),
                input_param: "pixel_values".to_string(),
                output_param: "image_embeds".to_string(),
                ..Default::default()
            }),
            _ => Err(AIProxyError::AIModelNotSupported),
        };
        Ok(ORTModel::Image(model_type?))
    }
}

impl ORTProvider {
    pub(crate) fn new() -> Self {
        Self {
            cache_location: None,
            cache_location_extension: PathBuf::from("huggingface"),
            supported_models: None,
            model: None,
        }
    }

    pub fn normalize(v: &[f32]) -> Vec<f32> {
        let norm = (v.iter().map(|val| val * val).sum::<f32>()).sqrt();
        let epsilon = 1e-12;

        // We add the super-small epsilon to avoid dividing by zero
        v.iter().map(|&val| val / (norm + epsilon)).collect()
    }

    pub fn batch_inference(&self, inputs: Vec<&ImageArray>) -> Result<Vec<Vec<f32>>, AIProxyError> {
        let model = match &self.model {
            Some(ORTModel::Image(model)) => model,
            _ => return Err(AIProxyError::AIModelNotSupported),
        };

        let array: Vec<Array<f32, Ix3>> = inputs.iter()
            .map(|image_arr| {
                let arr = image_arr.get_array();
                let mut arr = arr.mapv(f32::from);
                // Swapping axes from [rows, columns, channels] to [channels, rows, columns] for ONNX
                arr.swap_axes(1, 2);
                arr.swap_axes(0, 1);
                arr
            }).collect();

        let array_views: Vec<ArrayView<f32, Ix3>> = array.iter()
            .map(|arr| arr.view()).collect();

        let pixel_values_array = ndarray::stack(ndarray::Axis(0), &array_views).unwrap();
        match &model.session {
            Some(session) => {
                let session_inputs = ort::inputs![
                                model.input_param.as_str() => pixel_values_array.view(),
                            ].map_err(|_| AIProxyError::ModelProviderPreprocessingError)?;

                let outputs = session.run(session_inputs)
                    .map_err(|_| AIProxyError::ModelProviderRunInferenceError)?;
                let last_hidden_state_key = match outputs.len() {
                    1 => outputs.keys().next().unwrap(),
                    _ => model.output_param.as_str(),
                };

                let output_data = outputs[last_hidden_state_key]
                    .try_extract_tensor::<f32>()
                    .map_err(|_| AIProxyError::ModelProviderPostprocessingError)?;
                let embeddings: Vec<Vec<f32>> = output_data
                    .rows()
                    .into_iter()
                    .map(|row| ORTProvider::normalize(row.as_slice().unwrap()))
                    .collect();
                Ok(embeddings.to_owned())
            }
            None => Err(AIProxyError::AIModelNotInitialized)
        }
    }
}

impl ProviderTrait for ORTProvider {
    fn set_cache_location(&mut self, location: &Path) -> &mut Self {
        self.cache_location = Some(location.join(self.cache_location_extension.clone()));
        self
    }

    fn set_model(&mut self, model: &SupportedModels) -> &mut Self {
        self.supported_models = Some(*model);
        self
    }

    fn load_model(&mut self) -> &mut Self {
        let cache_location = self
            .cache_location
            .clone()
            .expect("Cache location not set.");
        let supported_model = self.supported_models.expect(
            &AIProxyError::AIModelNotInitialized.to_string());
        let ort_model = ORTModel::try_from(&supported_model).unwrap();

        let threads = available_parallelism()
            .expect("Could not find the threads")
            .get();

        let cache = Cache::new(cache_location);
        let api = ApiBuilder::from_cache(cache)
            .with_progress(true)
            .build()
            .unwrap();

        match ort_model {
            ORTModel::Image(ORTImageModel {
                weights_file,
                repo_name,
                input_param,
                output_param,
                ..
            }) => {
                let model_repo = api.model(repo_name.clone());
                let model_file_reference = model_repo
                    .get(&weights_file)
                    .unwrap_or_else(|_| panic!("Failed to retrieve {} ", weights_file));
                let executioners: Vec<ExecutionProviderDispatch> = Default::default();
                let session = Session::builder()
                    .unwrap()
                    .with_execution_providers(executioners)
                    .unwrap()
                    .with_optimization_level(GraphOptimizationLevel::Level3)
                    .unwrap()
                    .with_intra_threads(threads)
                    .unwrap()
                    .commit_from_file(model_file_reference)
                    .unwrap();
                self.model = Some(ORTModel::Image(ORTImageModel {
                    repo_name,
                    weights_file,
                    input_param,
                    output_param,
                    session: Some(session),
                }));
            }
        }
        self
    }

    fn get_model(&self) {
        let cache_location = self
            .cache_location
            .clone()
            .expect("Cache location not set.");
        let supported_model = self.supported_models.expect("A model has not been set.");
        let ort_model = ORTModel::try_from(&supported_model).unwrap();

        let cache = Cache::new(cache_location);
        let api = ApiBuilder::from_cache(cache)
            .with_progress(true)
            .build()
            .unwrap();

        match ort_model {
            ORTModel::Image(ORTImageModel {
                repo_name,
                weights_file,
                ..
            }) => {
                let model_repo = api.model(repo_name);
                model_repo
                    .get(&weights_file)
                    .unwrap_or_else(|_| panic!("Failed to retrieve {} ", weights_file));
                let preprocessor = model_repo.get("preprocessor_config.json");
                if preprocessor.is_err() {
                    log::warn!(
                        "Failed to retrieve preprocessor_config.json for model: {}",
                        supported_model
                    );
                }
            }
        }
    }

    fn run_inference(&self, inputs: &Vec<ModelInput>, action_type: &InputAction) -> Result<Vec<Vec<f32>>, AIProxyError> {
        let (string_inputs, image_inputs): (Vec<&String>, Vec<&ImageArray>) = inputs
            .par_iter().partition_map(|input| {
            match input {
                ModelInput::Text(value) => Either::Left(value),
                ModelInput::Image(value) => Either::Right(value),
            }
        });

        if !string_inputs.is_empty() {
            let store_input_type: AIStoreInputType = AIStoreInputType::RawString;
            let index_model_repr: Model = (&self.supported_models.expect(
                &AIProxyError::AIModelNotInitialized.to_string())).into();
            match action_type {
                InputAction::Query => {
                    return Err(AIProxyError::StoreQueryTypeMismatchError {
                        store_query_model_type: index_model_repr.to_string(),
                        storeinput_type: store_input_type.to_string(),
                    })
                }
                InputAction::Index => {
                    return Err(AIProxyError::StoreSetTypeMismatchError {
                        index_model_type: index_model_repr.to_string(),
                        storeinput_type: store_input_type.to_string(),
                    })
                }
            }
        }

        let batch_size = 16;
        let all_embeddings = image_inputs
            .par_chunks(batch_size)
            .map(|batch_inputs| {
                self.batch_inference(batch_inputs.to_vec())
            })
            .try_reduce(Vec::new, |mut accumulator, embeddings| {
                accumulator.extend(embeddings);
                Ok(accumulator)
            });

        return all_embeddings;
    }
}
