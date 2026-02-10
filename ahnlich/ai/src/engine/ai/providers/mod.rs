pub(crate) mod ort;
pub mod processors;

use crate::engine::ai::models::{InputAction, ModelInput};
use crate::engine::ai::providers::ort::ORTProvider;
use crate::error::AIProxyError;
use ahnlich_types::ai::execution_provider::ExecutionProvider;

use super::models::ModelResponse;

pub enum ModelProviders {
    ORT(ORTProvider),
}

#[async_trait::async_trait]
pub(crate) trait ProviderTrait: Send + Sync {
    async fn get_model(&self) -> Result<(), AIProxyError>;
    async fn run_inference(
        &self,
        input: ModelInput,
        action_type: &InputAction,
        execution_provider: Option<ExecutionProvider>,
    ) -> Result<Vec<ModelResponse>, AIProxyError>;
}
