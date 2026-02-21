use crate::engine::ai::models::{ModelInput, ModelResponse};
use crate::error::AIProxyError;
use ahnlich_types::ai::execution_provider::ExecutionProvider as AIExecutionProvider;
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Hash, Ord)]
pub(crate) enum ORTModality {
    Image,
    Text,
    Audio,
}

/// Trait for all ORT-based inference models (single-stage, multi-stage, etc.)
pub(crate) trait ORTInferenceModel: Send + Sync {
    /// Run inference on a batch of inputs
    /// execution_provider: None means use CPU
    /// model_params: Runtime parameters for the model (e.g., confidence_threshold)
    fn infer_batch(
        &self,
        input: ModelInput,
        execution_provider: Option<AIExecutionProvider>,
        model_params: &HashMap<String, String>,
    ) -> Pin<Box<dyn Future<Output = Result<Vec<ModelResponse>, AIProxyError>> + Send + '_>>;

    /// Get the batch size for this model
    fn batch_size(&self) -> usize;
}
