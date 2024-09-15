pub(crate) mod fastembed;

use std::hash::Hash;
use std::path::PathBuf;
use strum::EnumIter;

use crate::cli::server::SupportedModels;
use crate::engine::ai::providers::fastembed::{FastEmbedModel, FastEmbedProvider};

#[derive(Debug, EnumIter)]
pub enum  ModelProviders{
    FastEmbed(FastEmbedProvider),
}

pub trait ProviderTrait: std::fmt::Debug + Send + Sync {
    fn set_cache_location(&mut self, location: &PathBuf) -> &mut Self;
    fn set_model(&mut self, model: SupportedModels) -> &mut Self;
    fn load_model(&mut self) -> &mut Self;
    fn get_model(&self);
    fn run_inference(&self, input: &str) -> Vec<f32>;
}