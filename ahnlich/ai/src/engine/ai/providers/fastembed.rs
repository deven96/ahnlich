use fastembed::{EmbeddingModel, InitOptions, ImageEmbeddingModel, TextEmbedding};
use hf_hub::{api::sync::ApiBuilder, Cache};
use ahnlich_types::ai::AIModel;
use std::convert::TryFrom;
use std::path::PathBuf;
use crate::engine::ai::models::Model;


pub struct FastEmbedProvider {
    cache_location: PathBuf,
}

enum FastEmbedModel {
    Text(EmbeddingModel),
    Image(ImageEmbeddingModel),
}

#[derive(Debug)]
pub enum Error {
    ModelNotFound { reason: String},
}

impl TryFrom<&AIModel> for FastEmbedModel {
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
                FastEmbedModel::Text(model_type)
            },
            Model::Image { .. } => {
                let model_type = match model {
                    AIModel::Resnet50 => ImageEmbeddingModel::Resnet50,
                    AIModel::ClipVitB32 => ImageEmbeddingModel::ClipVitB32,
                    _ => return Err(Error::ModelNotFound {reason: "Model not supported".to_string()})
                };
                FastEmbedModel::Image(model_type)
            }
        };
        Ok(model_type)
    }
}


impl FastEmbedProvider {
    pub fn new(cache_location: PathBuf) -> Self {
        FastEmbedProvider {
            cache_location: cache_location
        }
    }

    pub fn download(&self, model: &AIModel) {
        let model_type = FastEmbedModel::try_from(model).expect("Model not found");

        if let FastEmbedModel::Text(model_type) = model_type {
            let cache = Cache::new(self.cache_location.clone());
            let api = ApiBuilder::from_cache(cache).with_progress(true).build().unwrap();
            let model_repo = api.model(model_type.to_string());
            let model_info = TextEmbedding::get_model_info(&model_type).unwrap();
            model_repo.get(model_info.model_file.as_str()).unwrap();
        }
    }
}