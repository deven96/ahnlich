use std::path::Path;
use std::sync::Arc;

use crate::cli::server::{ModelConfig, SupportedModels};
use crate::engine::ai::models::{ImageArray, InputAction};
/// The ModelManager is a wrapper around all the AI models running on various green threads. It
/// lets AIProxyTasks communicate with any model to receive immediate responses via a oneshot
/// channel
use crate::engine::ai::models::{Model, ModelInput};
use crate::engine::ai::providers::ModelProviders;
use crate::error::AIProxyError;
use ahnlich_types::ai::{AIModel, AIStoreInputType, ImageAction, PreprocessAction, StringAction};
use ahnlich_types::keyval::{StoreInput, StoreKey};
use fallible_collections::FallibleVec;
use moka::future::Cache;
use rayon::prelude::*;
use task_manager::Task;
use task_manager::TaskManager;
use task_manager::TaskState;
use tokio::sync::Mutex;
use tokio::sync::{mpsc, oneshot};
use tokio::time::Duration;
use tracing_opentelemetry::OpenTelemetrySpanExt;

type ModelThreadResponse = Result<Vec<StoreKey>, AIProxyError>;

struct ModelThreadRequest {
    // TODO: change this to a Vec of preprocessed input enum
    inputs: Vec<StoreInput>,
    response: oneshot::Sender<ModelThreadResponse>,
    preprocess_action: PreprocessAction,
    action_type: InputAction,
    trace_span: tracing::Span,
}

struct ModelThread {
    model: Model,
    request_receiver: Mutex<mpsc::Receiver<ModelThreadRequest>>,
}

impl ModelThread {
    fn new(
        supported_model: SupportedModels,
        cache_location: &Path,
        request_receiver: mpsc::Receiver<ModelThreadRequest>,
    ) -> Result<Self, AIProxyError> {
        let supported_model = &supported_model;
        let mut model: Model = (supported_model).into();
        model.setup_provider(cache_location);
        model.load()?;
        Ok(Self {
            request_receiver: Mutex::new(request_receiver),
            model,
        })
    }

    fn name(&self) -> String {
        format!("{:?}-model-thread", self.model.model_name())
    }

    #[tracing::instrument(skip(self, inputs))]
    fn input_to_response(
        &self,
        inputs: Vec<StoreInput>,
        process_action: PreprocessAction,
        action_type: InputAction,
    ) -> ModelThreadResponse {
        let mut response: Vec<_> = FallibleVec::try_with_capacity(inputs.len())?;
        let processed_inputs = self.preprocess_store_input(process_action, inputs)?;
        let mut store_key = self.model.model_ndarray(&processed_inputs, &action_type)?;
        response.append(&mut store_key);
        Ok(response)
    }

    #[tracing::instrument(skip(self, inputs))]
    pub(crate) fn preprocess_store_input(
        &self,
        process_action: PreprocessAction,
        inputs: Vec<StoreInput>,
    ) -> Result<Vec<ModelInput>, AIProxyError> {
        let preprocessed_inputs = inputs
            .into_par_iter()
            .try_fold(Vec::new, | mut accumulator, input| {
                let model_input = ModelInput::try_from(input)?;
                let processed_input = match (process_action, &model_input) {
                    (PreprocessAction::Image(image_action), ModelInput::Image(image_array)) => {
                        let output = self.process_image(image_array, image_action)?;
                        Ok(ModelInput::Image(output))
                    }
                    (PreprocessAction::RawString(string_action), ModelInput::Text(string)) => {
                        let output = self.preprocess_raw_string(string, string_action)?;
                        Ok(ModelInput::Text(output))
                    }
                    _ => {
                        let input_type: AIStoreInputType = (&model_input).into();
                        Err(AIProxyError::PreprocessingMismatchError {
                            input_type,
                            preprocess_action: process_action,
                        })
                    }
                }?;
            accumulator.push(processed_input);
            Ok::<Vec<ModelInput>, AIProxyError>(accumulator)
            })
            .try_reduce(Vec::new, |mut accumulator, mut item| {
            accumulator.append(&mut item);
            Ok(accumulator)
        })?;
        Ok(preprocessed_inputs)
    }
    #[tracing::instrument(skip(self, input))]
    fn preprocess_raw_string(
        &self,
        input: &str,
        string_action: StringAction,
    ) -> Result<String, AIProxyError> {
        let max_token_size = self.model.max_input_token().unwrap_or_else(|| {
            panic!(
                "`max_input_token()` is not supported for model: {:?}",
                self.model.model_name()
            )
        });

        if self.model.input_type() != AIStoreInputType::RawString {
            return Err(AIProxyError::TokenTruncationNotSupported);
        }

        let ModelProviders::FastEmbed(provider) = &self.model.provider else {
            return Err(AIProxyError::TokenTruncationNotSupported);
        };

        let tokens = provider.encode_str(input)?;

        if tokens.len() > max_token_size.into() {
            if let StringAction::ErrorIfTokensExceed = string_action {
                return Err(AIProxyError::TokenExceededError {
                    input_token_size: tokens.len(),
                    max_token_size: max_token_size.into(),
                });
            } else {
                let processed_input = provider.decode_tokens(tokens)?;
                return Ok(processed_input);
            }
        };
        Ok(input.to_owned())
    }

    #[tracing::instrument(skip(self, input))]
    fn process_image(
        &self,
        input: &ImageArray,
        image_action: ImageAction,
    ) -> Result<ImageArray, AIProxyError> {
        // process image, return error if max dimensions exceeded
        let dimensions = input.image_dim();

        let preprocess_mismatch = Err(AIProxyError::PreprocessingMismatchError {
            input_type: AIStoreInputType::Image,
            preprocess_action: PreprocessAction::Image(image_action),
        });

        let Some((expected_width, expected_height)) = self.model.expected_image_dimensions() else {
            return preprocess_mismatch;
        };

        let (width, height) = dimensions;

        if width != expected_width || height != expected_height {
            if let ImageAction::ErrorIfDimensionsMismatch = image_action {
                return Err(AIProxyError::ImageDimensionsMismatchError {
                    image_dimensions: (width.into(), height.into()),
                    expected_dimensions: (expected_width.into(), expected_height.into()),
                });
            } else {
                let input = input.resize(expected_width, expected_height)?;
                return Ok(input);
            }
        }

        Ok(input.clone())
    }
}

#[async_trait::async_trait]
impl Task for ModelThread {
    fn task_name(&self) -> String {
        self.name()
    }

    async fn run(&self) -> TaskState {
        if let Some(model_request) = self.request_receiver.lock().await.recv().await {
            // TODO actually service model request in here and return
            let ModelThreadRequest {
                inputs,
                response,
                preprocess_action,
                action_type,
                trace_span,
            } = model_request;
            let child_span = tracing::info_span!("model-thread-run", model = self.task_name());
            child_span.set_parent(trace_span.context());

            let responses = self.input_to_response(inputs, preprocess_action, action_type);
            if let Err(e) =
                response.send(responses)
            {
                log::error!("{} could not send response to channel {e:?}", self.name());
            }
            return TaskState::Continue;
        }
        TaskState::Break
    }
}

#[derive(Debug)]
pub struct ModelManager {
    models: Cache<SupportedModels, mpsc::Sender<ModelThreadRequest>>,
    supported_models: Vec<SupportedModels>,
    task_manager: Arc<TaskManager>,
    config: ModelConfig,
}

impl ModelManager {
    pub async fn new(
        model_config: ModelConfig,
        task_manager: Arc<TaskManager>,
    ) -> Result<Self, AIProxyError> {
        // TODO: Actually load the model at this point, with supported models and throw up errors,
        // also start up the various models with the task manager passed in
        let models = Cache::builder()
            .max_capacity(model_config.supported_models.len() as u64)
            .time_to_idle(Duration::from_secs(model_config.model_idle_time))
            .build();
        let model_manager = ModelManager {
            models,
            task_manager,
            supported_models: model_config.supported_models.to_vec(),
            config: model_config,
        };

        for model in &model_manager.supported_models {
            let _ = model_manager
                .models
                .try_get_with(*model, model_manager.try_initialize_model(model))
                .await
                .map_err(|err| AIProxyError::ModelInitializationError(err.to_string()))?;
        }
        Ok(model_manager)
    }

    #[tracing::instrument(skip(self))]
    async fn try_initialize_model(
        &self,
        model: &SupportedModels,
    ) -> Result<mpsc::Sender<ModelThreadRequest>, AIProxyError> {
        let (request_sender, request_receiver) = mpsc::channel(100);
        // There may be other things needed to load a model thread
        let model_thread =
            ModelThread::new(*model, &self.config.model_cache_location, request_receiver)?;
        let _ = &self.task_manager.spawn_task_loop(model_thread).await;
        Ok(request_sender)
    }

    #[tracing::instrument(skip(self, inputs))]
    pub async fn handle_request(
        &self,
        model: &AIModel,
        inputs: Vec<StoreInput>,
        preprocess_action: PreprocessAction,
        action_type: InputAction,
    ) -> Result<Vec<StoreKey>, AIProxyError> {
        let supported = model.into();

        if !self.supported_models.contains(&supported) {
            return Err(AIProxyError::AIModelNotInitialized);
        }
        let sender = self
            .models
            .try_get_with(supported, self.try_initialize_model(&supported))
            .await
            .map_err(|err| AIProxyError::ModelInitializationError(err.to_string()))?;

        let (response_tx, response_rx) = oneshot::channel();
        let request = ModelThreadRequest {
            inputs,
            response: response_tx,
            preprocess_action,
            action_type,
            trace_span: tracing::Span::current(),
        };
        // TODO: Add potential timeouts for send and recieve in case threads are unresponsive
        if sender.send(request).await.is_ok() {
            response_rx
                .await
                .map_err(|e| e.into())
                .and_then(|inner| inner)
        } else {
            return Err(AIProxyError::AIModelThreadSendError);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_model_manager_setup_works() {
        let supported_models = vec![
            SupportedModels::AllMiniLML6V2,
            SupportedModels::AllMiniLML12V2,
        ];

        let task_manager = Arc::new(TaskManager::new());
        let time_to_idle: u64 = 5;
        let model_config = ModelConfig {
            supported_models: supported_models.clone(),
            model_idle_time: time_to_idle,
            ..Default::default()
        };
        let model_manager = ModelManager::new(model_config, task_manager).await.unwrap();

        for model in supported_models {
            let recreated_model = model_manager.models.get(&model).await;
            assert!(recreated_model.is_some());
        }
    }

    #[tokio::test]
    async fn test_model_manager_refreshes_evicted_model() {
        let supported_models = vec![
            SupportedModels::AllMiniLML6V2,
            SupportedModels::AllMiniLML12V2,
        ];
        let sample_ai_model = AIModel::AllMiniLML6V2;
        let sample_supported_model: SupportedModels = (&sample_ai_model).into();

        let task_manager = Arc::new(TaskManager::new());
        let time_to_idle: u64 = 1;
        let model_config = ModelConfig {
            supported_models: supported_models.clone(),
            model_idle_time: time_to_idle,
            ..Default::default()
        };
        let model_manager = ModelManager::new(model_config, task_manager).await.unwrap();
        let _ = tokio::time::sleep(Duration::from_secs(2)).await;

        let evicted_model = model_manager.models.get(&sample_supported_model).await;

        let inputs = vec![StoreInput::RawString(String::from("Hello"))];
        let action = PreprocessAction::RawString(StringAction::TruncateIfTokensExceed);
        let _ = model_manager
            .handle_request(&sample_ai_model, inputs, action, InputAction::Query)
            .await
            .unwrap();
        let recreated_model = model_manager.models.get(&sample_supported_model).await;

        assert!(evicted_model.is_none());
        assert!(recreated_model.is_some());
    }
}
