pub(crate) mod fastembed;

use std::hash::Hash;
use std::path::PathBuf;
use strum::EnumIter;

use crate::cli::server::SupportedModels;
use crate::engine::ai::providers::fastembed::FastEmbedProvider;

#[derive(Debug, EnumIter, PartialEq, Hash, Eq, Clone)]
pub enum  ModelProviders{
    FastEmbed(FastEmbedProvider),
}

pub trait ProviderTrait: std::fmt::Debug + Send + Sync + Hash + PartialEq {
    fn set_cache_location(&mut self, location: PathBuf) -> &mut Self;
    fn set_model(&mut self, model: SupportedModels) -> &mut Self;
    fn load_model(&mut self) -> &mut Self;
    fn get_model(&self);
}