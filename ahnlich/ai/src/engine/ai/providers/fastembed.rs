use crate::cli::server::SupportedModels;
use crate::engine::ai::models::Model;
use crate::engine::ai::providers::ProviderTrait;
use fastembed::{EmbeddingModel, ImageEmbedding, ImageEmbeddingModel, InitOptions, TextEmbedding};
use hf_hub::{api::sync::ApiBuilder, Cache};
use std::convert::TryFrom;
use std::fmt;
use std::path::PathBuf;

#[derive(Default)]
pub struct FastEmbedProvider {
    cache_location: Option<PathBuf>,
    cache_location_extension: PathBuf,
    supported_models: Option<SupportedModels>,
    pub model: Option<FastEmbedModel>,
}

pub enum FastEmbedModelType {
    Text(EmbeddingModel),
    Image(ImageEmbeddingModel),
}

impl fmt::Debug for FastEmbedProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // You can customize the output here. For example, if you can't print the `TextEmbedding`,
        // you might choose to skip it or provide a placeholder message.
        f.debug_struct("FastEmbedProvider")
            .field("cache_location", &self.cache_location)
            .field("cache_location_extension", &self.cache_location_extension)
            .field("supported_models", &self.supported_models) // Placeholder
            .finish()
    }
}

pub enum FastEmbedModel {
    Text(TextEmbedding),
    Image(ImageEmbedding),
}

#[derive(Debug)]
pub enum Error {
    ModelNotFound { reason: String },
}

// TODO (HAKSOAT): Remove this tryfrom.
impl TryFrom<&SupportedModels> for FastEmbedModelType {
    type Error = Error;

    fn try_from(model: &SupportedModels) -> Result<Self, Self::Error> {
        let model_modality: Model = model.into();
        let model_type = match model_modality {
            Model::Text { .. } => {
                let model_type = match model {
                    SupportedModels::AllMiniLML6V2 => EmbeddingModel::AllMiniLML6V2,
                    SupportedModels::AllMiniLML12V2 => EmbeddingModel::AllMiniLML12V2,
                    SupportedModels::BGEBaseEnV15 => EmbeddingModel::BGEBaseENV15,
                    SupportedModels::BGELargeEnV15 => EmbeddingModel::BGELargeENV15,
                    _ => {
                        return Err(Error::ModelNotFound {
                            reason: "Model not supported".to_string(),
                        })
                    }
                };
                FastEmbedModelType::Text(model_type)
            }
            Model::Image { .. } => {
                let model_type = match model {
                    SupportedModels::Resnet50 => ImageEmbeddingModel::Resnet50,
                    SupportedModels::ClipVitB32 => ImageEmbeddingModel::ClipVitB32,
                    _ => {
                        return Err(Error::ModelNotFound {
                            reason: "Model not supported".to_string(),
                        })
                    }
                };
                FastEmbedModelType::Image(model_type)
            }
        };
        Ok(model_type)
    }
}

impl FastEmbedProvider {
    pub(crate) fn new() -> Self {
        FastEmbedProvider {
            cache_location: None,
            cache_location_extension: PathBuf::from("huggingface".to_string()),
            supported_models: None,
            model: None,
        }
    }
}

impl ProviderTrait for FastEmbedProvider {
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
        let model_type = self
            .supported_models
            .clone()
            .expect("Model has not been set.");
        let model_type = FastEmbedModelType::try_from(&model_type).expect(
            format!(
                "This provider does not support the model: {:?}.",
                model_type
            )
            .as_str(),
        );
        let cache_location = self
            .cache_location
            .clone()
            .expect("Cache location not set.");

        if let FastEmbedModelType::Text(model_type) = model_type {
            let model =
                TextEmbedding::try_new(InitOptions::new(model_type).with_cache_dir(cache_location))
                    .unwrap();
            self.model = Some(FastEmbedModel::Text(model));
        }

        self
    }

    fn get_model(&self) {
        let model_type = self
            .supported_models
            .clone()
            .expect("A model has not been set.");
        let model_type = FastEmbedModelType::try_from(&model_type).expect(
            format!(
                "This provider does not support the model: {:?}.",
                model_type
            )
            .as_str(),
        );
        let cache_location = self
            .cache_location
            .clone()
            .expect("Cache location not set.");

        if let FastEmbedModelType::Text(model_type) = model_type {
            let cache = Cache::new(cache_location);
            let api = ApiBuilder::from_cache(cache)
                .with_progress(true)
                .build()
                .unwrap();
            let model_repo = api.model(model_type.to_string());
            let model_info = TextEmbedding::get_model_info(&model_type).unwrap();
            model_repo.get(model_info.model_file.as_str()).unwrap();
        }
    }

    fn run_inference(&self, input: &str) -> Vec<f32> {
        let input = vec![input];
        if let Some(fastembed_model) = &self.model {
            match fastembed_model {
                FastEmbedModel::Text(model) => {
                    let response = model.embed(input, None).expect("Could not run inference.");
                    let response: Vec<f32> = response
                        .get(0)
                        .expect("Response embedding is empty")
                        .to_owned()
                        .into();
                    response
                }
                FastEmbedModel::Image(model) => {
                    let response = model.embed(input, None).expect("Could not run inference.");
                    let response: Vec<f32> = response
                        .get(0)
                        .expect("Response embedding is empty")
                        .to_owned()
                        .into();
                    response
                }
            }
        } else {
            panic!("Model has not been loaded.");
        }
    }
}
