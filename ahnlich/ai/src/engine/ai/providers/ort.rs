use ahnlich_types::keyval::StoreInput;
use crate::cli::server::SupportedModels;
use crate::engine::ai::models::{InputAction, Model};
use crate::engine::ai::providers::ProviderTrait;
use crate::error::AIProxyError;
use ort::{ExecutionProviderDispatch, GraphOptimizationLevel, Session};
use hf_hub::{api::sync::ApiBuilder, Cache};
use log;
use std::convert::TryFrom;
use std::fmt;
use std::path::PathBuf;
use std::thread::available_parallelism;
use tracing::warn;

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

pub struct ORTModel {
    session: Session
}

pub struct ORTModelType {
    repo_name: String,
    weights_file: String
}

impl TryFrom<&SupportedModels> for ORTModelType {
    type Error = AIProxyError;

    fn try_from(model: &SupportedModels) -> Result<Self, Self::Error> {
        let model_type = match model {
            SupportedModels::Resnet50 => Ok(ORTModelType {
                repo_name: "Qdrant/resnet50-onnx".to_string(),
                weights_file: "model.onnx".to_string()
            }),
            SupportedModels::ClipVitB32 => Ok(ORTModelType {
                repo_name: "Qdrant/clip-ViT-B-32-vision".to_string(),
                weights_file: "model.onnx".to_string()
            }),
            _ => Err(AIProxyError::AIModelNotSupported)
        };
        return model_type
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
}


impl ProviderTrait for ORTProvider {
    fn set_cache_location(&mut self, location: &PathBuf) -> &mut Self {
        let mut cache_location = location.clone();
        cache_location.push(self.cache_location_extension.clone());
        self.cache_location = Some(cache_location);
        self
    }

    fn set_model(&mut self, model: SupportedModels) -> &mut Self {
        self.supported_models = Some(model);
        self
    }

    fn load_model(&mut self) -> &mut Self {
        let cache_location = self
            .cache_location
            .clone()
            .expect("Cache location not set.");
        let supported_model = self.supported_models.expect("A model has not been set.");
        let ort_model = ORTModelType::try_from(&supported_model).unwrap();

        let threads = available_parallelism().expect("Check again").get();

        let cache = Cache::new(cache_location);
        let api = ApiBuilder::from_cache(cache)
            .with_progress(true)
            .build()
            .unwrap();

        if let ORTModelType{weights_file, repo_name} = ort_model {
            let model_repo = api.model(repo_name);
            let model_file_reference = model_repo.get(&weights_file)
                .unwrap_or_else(|_| panic!("Failed to retrieve {} ", weights_file));
            let executioners: Vec<ExecutionProviderDispatch> = Default::default();
            let session = Session::builder().unwrap()
                .with_execution_providers(executioners).unwrap()
                .with_optimization_level(GraphOptimizationLevel::Level3).unwrap()
                .with_intra_threads(threads).unwrap()
                .commit_from_file(model_file_reference).unwrap();
            self.model = Some(ORTModel { session });
        };
        self
    }

    fn get_model(&self) {
        let cache_location = self
            .cache_location
            .clone()
            .expect("Cache location not set.");
        let supported_model = self.supported_models.expect("A model has not been set.");
        let ort_model = ORTModelType::try_from(&supported_model).unwrap();

        let cache = Cache::new(cache_location);
        let api = ApiBuilder::from_cache(cache)
            .with_progress(true)
            .build()
            .unwrap();

        if let ORTModelType { repo_name, weights_file } = ort_model {
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

    fn run_inference(&self, input: &StoreInput, action_type: InputAction) -> Vec<f32> {
        todo!()
    }
}