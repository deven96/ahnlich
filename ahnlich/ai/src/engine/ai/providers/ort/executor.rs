use moka::future::Cache as MokaCache;
use ort::Session;

use crate::error::AIProxyError;
use strum::IntoEnumIterator;

use super::{InnerAIExecutionProvider, register_provider};
use std::path::PathBuf;
use std::sync::Arc;
use std::thread::available_parallelism;

// Handles dynamically retrieving a session for an execution provider or creating the session if
// not exists
pub struct ExecutorWithSessionCache {
    cache: MokaCache<InnerAIExecutionProvider, Arc<Session>>,
    model_file_reference: PathBuf,
    session_profiling: bool,
}

impl ExecutorWithSessionCache {
    #[tracing::instrument]
    pub fn new(model_file_reference: PathBuf, session_profiling: bool) -> Self {
        Self {
            model_file_reference,
            session_profiling,
            cache: MokaCache::new(InnerAIExecutionProvider::iter().count() as u64),
        }
    }

    #[tracing::instrument(skip(self))]
    async fn inner_get_with(
        &self,
        execution_provider: InnerAIExecutionProvider,
    ) -> Result<Arc<Session>, AIProxyError> {
        let threads = available_parallelism()
            .map_err(|e| AIProxyError::APIBuilderError(e.to_string()))?
            .get();
        let mut session_builder = Session::builder()?.with_intra_threads(threads)?;

        if self.session_profiling {
            session_builder = session_builder.with_profiling("profiling.json")?;
        }
        register_provider(execution_provider, &session_builder)?;
        Ok(Arc::new(
            session_builder.commit_from_file(self.model_file_reference.clone())?,
        ))
    }

    #[tracing::instrument(skip(self))]
    pub async fn try_get_with(
        &self,
        execution_provider: InnerAIExecutionProvider,
    ) -> Result<Arc<Session>, AIProxyError> {
        self.cache
            .try_get_with(execution_provider, self.inner_get_with(execution_provider))
            .await
            .map_err(|e| (*e).clone())
    }
}
