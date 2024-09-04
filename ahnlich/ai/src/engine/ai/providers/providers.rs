use std::path::PathBuf;
use strum::{EnumIter, IntoEnumIterator};
use ahnlich_types::ai::AIModel;
use crate::engine::ai::providers::fastembed::{FastEmbedProvider, FastEmbedModelType};
use crate::engine::ai::providers::ProviderTrait;


#[derive(EnumIter)]
pub enum ModelProviderTypes{
    FastEmbed,
}

pub struct ModelProvider {
    model: AIModel,
    provider: Box<dyn ProviderTrait>
}

impl ModelProvider{
    pub fn new(model: &AIModel, cache_location: PathBuf) -> Self {
        let mut has_provider: Option<Box<dyn ProviderTrait>> = None;
        for ptype in ModelProviderTypes::iter() {
            match ptype {
                ModelProviderTypes::FastEmbed => {
                    if FastEmbedModelType::try_from(model).is_ok() {
                        let fastembed = FastEmbedProvider::new();
                        has_provider = Some(Box::new(fastembed));
                    }
                }
            }
        }

        let mut provider = match has_provider {
            Some(value) => value,
            _ => panic!("No provider found for model")
        };

        provider.set_cache_location(cache_location);
        provider.set_model(model.clone());

        ModelProvider {
            provider: provider,
            model: model.clone()
        }
    }

    pub fn load(&mut self) {
        self.provider.load_model();
    }

    pub fn download(&self) {
        self.provider.download_model();
    }
}
