use crate::engine::ai::models::{ModelInput, ModelResponse};
use crate::error::AIProxyError;
use ahnlich_types::ai::execution_provider::ExecutionProvider as AIExecutionProvider;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Hash, Ord)]
pub(crate) enum ORTModality {
    Image,
    Text,
}

/// Trait for all ORT-based inference models (single-stage, multi-stage, etc.)
pub(crate) trait ORTInferenceModel: Send + Sync {
    /// Run inference on a batch of inputs
    /// execution_provider: None means use CPU
    fn infer_batch(
        &self,
        input: ModelInput,
        execution_provider: Option<AIExecutionProvider>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ModelResponse>, AIProxyError>> + Send + '_>>;

    /// Get the batch size for this model
    fn batch_size(&self) -> usize;
}
