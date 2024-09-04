pub mod providers;
mod fastembed;

use std::path::PathBuf;
use ahnlich_types::ai::AIModel;
pub use crate::engine::ai::providers::providers::ModelProvider;

pub trait ProviderTrait {
    fn set_cache_location(&mut self, location: PathBuf) -> &mut dyn ProviderTrait;
    fn set_model(&mut self, model: AIModel) -> &mut dyn ProviderTrait;
    fn load_model(&mut self) -> &mut dyn ProviderTrait;
    fn download_model(&self);
}