use ahnlich_types::ai::AIStoreInputType;
use crate::cli::server::SupportedModels;
use crate::engine::ai::models::{InputAction, ModelInput, Model};
use crate::engine::ai::providers::ProviderTrait;
use crate::error::AIProxyError;
use ort::{ExecutionProviderDispatch, GraphOptimizationLevel, Session};
use hf_hub::{api::sync::ApiBuilder, Cache};
use log;

use std::default::Default;
use std::convert::TryFrom;
use std::fmt;
use std::path::PathBuf;
use std::thread::available_parallelism;


#[derive(Default)]
pub struct ORTProvider {
    cache_location: Option<PathBuf>,
    cache_location_extension: PathBuf,
    supported_models: Option<SupportedModels>,
    model: Option<ORTModel>,
}


impl fmt::Debug for ORTProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CandleProvider")
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
    output_param: String
}

pub enum ORTModel {
    Image(ORTImageModel)
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
            _ => Err(AIProxyError::AIModelNotSupported)
        };
        return Ok(ORTModel::Image(model_type?));
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
}


impl ProviderTrait for ORTProvider {
    fn set_cache_location(&mut self, location: &PathBuf) -> &mut Self {
        let mut cache_location = location.clone();
        cache_location.push(self.cache_location_extension.clone());
        self.cache_location = Some(cache_location);
        self
    }

    fn set_model(&mut self, model: &SupportedModels) -> &mut Self {
        self.supported_models = Some(model.clone());
        self
    }

    fn load_model(&mut self) -> &mut Self {
        let cache_location = self
            .cache_location
            .clone()
            .expect("Cache location not set.");
        let supported_model = self.supported_models.expect("A model has not been set.");
        let ort_model = ORTModel::try_from(&supported_model).unwrap();

        let threads = available_parallelism().expect("Could not find the threads").get();

        let cache = Cache::new(cache_location);
        let api = ApiBuilder::from_cache(cache)
            .with_progress(true)
            .build()
            .unwrap();

        match ort_model {
            ORTModel::Image(ORTImageModel {weights_file, repo_name, input_param,
                                output_param, ..}) => {
                let model_repo = api.model(repo_name.clone());
                let model_file_reference = model_repo.get(&weights_file)
                    .unwrap_or_else(|_| panic!("Failed to retrieve {} ", weights_file));
                let executioners: Vec<ExecutionProviderDispatch> = Default::default();
                let session = Session::builder().unwrap()
                    .with_execution_providers(executioners).unwrap()
                    .with_optimization_level(GraphOptimizationLevel::Level3).unwrap()
                    .with_intra_threads(threads).unwrap()
                    .commit_from_file(model_file_reference).unwrap();
                self.model = Some(ORTModel::Image(ORTImageModel {
                    repo_name,
                    weights_file,
                    input_param,
                    output_param,
                    session: Some(session)
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
            ORTModel::Image(ORTImageModel { repo_name, weights_file, .. }) => {
                let model_repo = api.model(repo_name);
                model_repo
                    .get(&weights_file)
                    .unwrap_or_else(|_| panic!("Failed to retrieve {} ", weights_file));
                let preprocessor = model_repo.get("preprocessor_config.json");
                if preprocessor.is_err() {
                    log::warn!("Failed to retrieve preprocessor_config.json for model: {}", supported_model);
                }
            }
        }
    }

    fn run_inference(&self, input: &ModelInput, action_type: &InputAction) -> Vec<f32> {
        if let Some(ORTModel::Image(ORTImageModel{session, input_param,
                                        output_param, ..})) = &self.model {
            let image = match input {
                ModelInput::Image(image) => image.clone(),
                _ => {
                    let store_input_type: AIStoreInputType = input.into();
                    let index_model_repr: Model = (&self.supported_models.unwrap()).into();
                    match action_type {
                        InputAction::Query => {
                            panic!("{}", AIProxyError::StoreQueryTypeMismatchError {
                                store_query_model_type: index_model_repr.to_string(),
                                storeinput_type: store_input_type.to_string(),
                            })
                        },
                        InputAction::Index => {
                            panic!("{}", AIProxyError::StoreSetTypeMismatchError {
                                index_model_type: index_model_repr.to_string(),
                                storeinput_type: store_input_type.to_string(),
                            })
                        },
                    }
                }
            };

            let image_arr = image.get_array();
            let mut image_arr = image_arr.mapv(f32::from);
            image_arr.swap_axes(1, 2);
            image_arr.swap_axes(0, 1);

            let array = vec![image_arr.view()];
            let pixel_values_array = ndarray::stack(ndarray::Axis(0), &array).unwrap();

            match session {
                Some(session) => {
                    let session_inputs = ort::inputs![
                        input_param.as_str() => pixel_values_array.view(),
                    ].unwrap();
                    let outputs = session.run(session_inputs).unwrap();
                    let last_hidden_state_key = match outputs.len() {
                        1 => outputs.keys().next().unwrap(),
                        _ => output_param.as_str(),
                    };
                    let output_data = outputs[last_hidden_state_key].try_extract_tensor::<f32>().unwrap();
                    let embeddings: Vec<Vec<f32>> = output_data
                        .rows()
                        .into_iter()
                        .map(|row| ORTProvider::normalize(row.as_slice().unwrap()))
                        .collect();
                    return embeddings.get(0).unwrap().to_owned();
                }
                None => panic!("{}", AIProxyError::AIModelNotInitialized),
            }
        } else {
            panic!("{}", AIProxyError::AIModelNotInitialized);
        }
    }
}