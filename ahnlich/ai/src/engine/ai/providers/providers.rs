use std::path::PathBuf;
use ahnlich_types::ai::AIModel;
use crate::engine::ai::providers::fastembed::FastEmbedProvider;


pub enum ModelProviders {
    FastEmbed,
}

impl ModelProviders {
    pub fn download(&self, model: &AIModel, cache_location: PathBuf) {
        match self {
            ModelProviders::FastEmbed => {
                let prov = FastEmbedProvider::new(cache_location);
                prov.download(model);
            },
        }
    }

    pub fn get_embeddings(&self) {
        match self {
            ModelProviders::FastEmbed => (),
        }
    }
}