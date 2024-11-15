pub(crate) mod fastembed;
pub(crate) mod ort;
mod ort_text_helper;
mod ort_helper;
mod processors;

use crate::cli::server::SupportedModels;
use crate::engine::ai::models::{InputAction, ModelInput};
use crate::engine::ai::providers::fastembed::FastEmbedProvider;
use crate::engine::ai::providers::ort::ORTProvider;
use crate::error::AIProxyError;
use ahnlich_types::keyval::StoreKey;
use std::path::Path;
use strum::EnumIter;


#[derive(Debug, EnumIter)]
pub enum ModelProviders {
    FastEmbed(FastEmbedProvider),
    ORT(ORTProvider),
}

pub trait ProviderTrait: std::fmt::Debug + Send + Sync {
    fn set_cache_location(&mut self, location: &Path);
    fn set_model(&mut self, model: &SupportedModels);
    fn load_model(&mut self) -> Result<(), AIProxyError>;
    fn get_model(&self) -> Result<(), AIProxyError>;
    fn run_inference(
        &self,
        input: Vec<ModelInput>,
        action_type: &InputAction,
    ) -> Result<Vec<StoreKey>, AIProxyError>;
}

pub trait TextPreprocessorTrait {
    fn encode_str(&self, text: &str) -> Result<Vec<u32>, AIProxyError>;
    fn decode_tokens(&self, tokens: Vec<u32>) -> Result<String, AIProxyError>;
}
