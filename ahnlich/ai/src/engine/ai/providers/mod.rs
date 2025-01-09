pub(crate) mod ort;
mod ort_helper;
pub mod processors;

use crate::engine::ai::models::{InputAction, ModelInput};
use crate::engine::ai::providers::ort::ORTProvider;
use crate::error::AIProxyError;
use ahnlich_types::keyval::StoreKey;

pub enum ModelProviders {
    ORT(ORTProvider),
}

pub trait ProviderTrait: Send + Sync {
    fn get_model(&self) -> Result<(), AIProxyError>;
    fn run_inference(
        &self,
        input: ModelInput,
        action_type: &InputAction,
    ) -> Result<Vec<StoreKey>, AIProxyError>;
}
