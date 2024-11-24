pub(crate) mod ort;
mod ort_helper;
mod processors;

use crate::cli::server::SupportedModels;
use crate::engine::ai::models::{InputAction, ModelInput};
use crate::engine::ai::providers::ort::ORTProvider;
use crate::error::AIProxyError;
use ahnlich_types::keyval::StoreKey;
use std::path::Path;
use strum::EnumIter;


#[derive(Debug, EnumIter)]
pub enum ModelProviders {
    ORT(ORTProvider),
}

pub trait ProviderTrait: std::fmt::Debug + Send + Sync {
    fn set_cache_location(&mut self, location: &Path);
    fn set_model(&mut self, model: &SupportedModels);
    fn load_model(&mut self) -> Result<(), AIProxyError>;
    fn get_model(&self) -> Result<(), AIProxyError>;
    fn run_inference(
        &self,
        input: ModelInput,
        action_type: &InputAction,
    ) -> Result<Vec<StoreKey>, AIProxyError>;
}
