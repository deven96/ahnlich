use fastembed::{EmbeddingModel, InitOptions, ImageEmbeddingModel, TextEmbedding, ImageEmbedding};
use hf_hub::{api::sync::ApiBuilder, Cache};
use ahnlich_types::ai::AIModel;
use std::convert::TryFrom;
use std::path::PathBuf;
use crate::engine::ai::models::Model;
use crate::engine::ai::providers::ProviderTrait;


pub struct FastEmbedProvider {
    cache_location: Option<PathBuf>,
    cache_location_extension: PathBuf,
    ai_model_type: Option<AIModel>,
    pub model: Option<FastEmbedModel>,
}


pub enum FastEmbedModelType {
    Text(EmbeddingModel),
    Image(ImageEmbeddingModel),
}


enum FastEmbedModel {
    Text(TextEmbedding),
    Image(ImageEmbedding),
}


#[derive(Debug)]
pub enum Error {
    ModelNotFound { reason: String},
}


impl TryFrom<&AIModel> for FastEmbedModelType {
    type Error = Error;

    fn try_from(model: &AIModel) -> Result<Self, Self::Error> {
        let model_modality: Model = model.into();
        let model_type = match model_modality {
            Model::Text { .. } => {
                let model_type = match model {
                    AIModel::AllMiniLML6V2 => EmbeddingModel::AllMiniLML6V2,
                    AIModel::AllMiniLML12V2 => EmbeddingModel::AllMiniLML12V2,
                    AIModel::BGEBaseEnV15 => EmbeddingModel::BGEBaseENV15,
                    AIModel::BGELargeEnV15 => EmbeddingModel::BGELargeENV15,
                    _ => return Err(Error::ModelNotFound {reason: "Model not supported".to_string()})
                };
                FastEmbedModelType::Text(model_type)
            },
            Model::Image { .. } => {
                let model_type = match model {
                    AIModel::Resnet50 => ImageEmbeddingModel::Resnet50,
                    AIModel::ClipVitB32 => ImageEmbeddingModel::ClipVitB32,
                    _ => return Err(Error::ModelNotFound {reason: "Model not supported".to_string()})
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
            ai_model_type: None,
            model: None,
        }
    }
}

impl ProviderTrait for FastEmbedProvider {
    fn set_cache_location(&mut self, location: PathBuf) -> &mut dyn ProviderTrait {
        let mut cache_location = location.clone();
        cache_location.push(self.cache_location_extension.clone());
        self.cache_location = Some(cache_location);
        self
    }

    fn set_model(&mut self, model: AIModel) -> &mut dyn ProviderTrait {
        self.ai_model_type = Some(model);
        self
    }

    fn load_model(&mut self) -> &mut dyn ProviderTrait {
        let model_type = self.ai_model_type.clone().expect("Model has not been set.");
        let model_type = FastEmbedModelType::try_from(&model_type)
            .expect(format!("This provider does not support the model: {:?}.", model_type).as_str());
        let cache_location = self.cache_location.clone().expect("Cache location not set.");

        if let FastEmbedModelType::Text(model_type) = model_type {
            let model = TextEmbedding::try_new(
                InitOptions::new(EmbeddingModel::AllMiniLML6V2)
                    .with_cache_dir(cache_location),
            ).unwrap();
            self.model = Some(FastEmbedModel::Text(model));
        }

        self
    }

    fn download_model(&self) {
        let model_type = self.ai_model_type.clone().expect("A model has not been set.");
        let model_type = FastEmbedModelType::try_from(&model_type)
            .expect(format!("This provider does not support the model: {:?}.", model_type).as_str());
        let cache_location = self.cache_location.clone().expect("Cache location not set.");

        if let FastEmbedModelType::Text(model_type) = model_type {
            let cache = Cache::new(cache_location);
            let api = ApiBuilder::from_cache(cache).with_progress(true).build().unwrap();
            let model_repo = api.model(model_type.to_string());
            let model_info = TextEmbedding::get_model_info(&model_type).unwrap();
            model_repo.get(model_info.model_file.as_str()).unwrap();
        }
    }
}
