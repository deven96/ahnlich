use crate::cli::server::SupportedModels;
use crate::engine::ai::models::{InputAction, ImageArray, Model, ModelInput};
use crate::engine::ai::providers::ProviderTrait;
use crate::error::AIProxyError;
use ahnlich_types::ai::AIStoreInputType;
use hf_hub::{api::sync::ApiBuilder, Cache};
use ort::Session;
use rayon::prelude::*;
use rayon::iter::Either;

use std::convert::TryFrom;
use std::default::Default;
use std::fmt;
use std::path::{Path, PathBuf};
use std::thread::available_parallelism;
use ndarray::{Array, Array1, ArrayView, Axis, Ix3};
use ahnlich_types::keyval::StoreKey;

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
        let norm = (v.par_iter().map(|val| val * val).sum::<f32>()).sqrt();
        let epsilon = 1e-12;

        // We add the super-small epsilon to avoid dividing by zero
        v.par_iter().map(|&val| val / (norm + epsilon)).collect()
    }

    pub fn batch_inference(&self, inputs: &[&ImageArray]) -> Result<Vec<StoreKey>, AIProxyError> {
        let model = match &self.model {
            Some(ORTModel::Image(model)) => model,
            _ => return Err(AIProxyError::AIModelNotSupported),
        };

        let array: Vec<Array<f32, Ix3>> = inputs.par_iter()
            .map(|image_arr| {
                let arr = image_arr.get_array();
                let mut arr = arr.mapv(f32::from);
                // Swapping axes from [rows, columns, channels] to [channels, rows, columns] for ONNX
                arr.swap_axes(1, 2);
                arr.swap_axes(0, 1);
                arr
            })
            .collect();

        // TODO: Figure how to avoid this second par_iter.
        let array_views: Vec<ArrayView<f32, Ix3>> = array.par_iter()
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
                let store_keys = output_data
                    .axis_iter(Axis(0))
                    .into_par_iter()
                    .map(|row| {
                        let embeddings = ORTProvider::normalize(row.as_slice().unwrap());
                        StoreKey(<Array1<f32>>::from(embeddings))
                    })
                    .collect();
                Ok(store_keys)
            }
            None => Err(AIProxyError::AIModelNotInitialized)
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
                input_param,
                output_param,
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
                    input_param,
                    output_param,
                    session: Some(session),
                }));
            }
        }
        Ok(())
    }

    fn get_model(&self) -> Result<(), AIProxyError> {
        let Some(cache_location) = self.cache_location.clone() else {
            return Err(AIProxyError::CacheLocationNotInitiailized);
        };
        let supported_model = self.supported_models
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
            }
        }
    }

    fn run_inference(&self, inputs: &[ModelInput], action_type: &InputAction) -> Result<Vec<StoreKey>, AIProxyError> {
        let (string_inputs, image_inputs): (Vec<&String>, Vec<&ImageArray>) = inputs
            .par_iter().partition_map(|input| {
            match input {
                ModelInput::Text(value) => Either::Left(value),
                ModelInput::Image(value) => Either::Right(value),
            }
        });

        if !string_inputs.is_empty() {
            let store_input_type: AIStoreInputType = AIStoreInputType::RawString;
            let Some(index_model_repr) = self.supported_models else {
                return Err(AIProxyError::AIModelNotInitialized);
            };
            let index_model_repr: Model = (&index_model_repr).into();
            return Err(AIProxyError::StoreTypeMismatchError {
                action: *action_type,
                index_model_type: index_model_repr.input_type(),
                storeinput_type: store_input_type,
            });
        }

        let batch_size = 16;
        let store_keys = image_inputs
            .chunks(batch_size)
            .try_fold(Vec::new(), |mut accumulator, batch_inputs|{
                accumulator.extend(self.batch_inference(batch_inputs)?);
                Ok(accumulator)
            });

        store_keys
    }
}
