pub(crate) mod fastembed;
pub(crate) mod ort;

use crate::cli::server::SupportedModels;
use crate::engine::ai::models::{InputAction, ModelInput};
use crate::engine::ai::providers::fastembed::FastEmbedProvider;
use crate::engine::ai::providers::ort::ORTProvider;
use std::path::Path;
use strum::EnumIter;

#[derive(Debug, EnumIter)]
pub enum ModelProviders {
    FastEmbed(FastEmbedProvider),
    ORT(ORTProvider),
}

pub trait ProviderTrait: std::fmt::Debug + Send + Sync {
    fn set_cache_location(&mut self, location: &Path) -> &mut Self;
    fn set_model(&mut self, model: &SupportedModels) -> &mut Self;
    fn load_model(&mut self) -> &mut Self;
    fn get_model(&self);
    fn run_inference(&self, input: &ModelInput, action_type: &InputAction) -> Vec<f32>;
}
