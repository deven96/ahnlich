use std::path::Path;
use std::sync::Arc;
use std::sync::OnceLock;

use crate::cli::server::{ModelConfig, SupportedModels};
use crate::engine::ai::models::{ImageArray, InputAction, ModelResponse};
/// The ModelManager is a wrapper around all the AI models running on various green threads. It
/// lets AIProxyTasks communicate with any model to receive immediate responses via a oneshot
/// channel
use crate::engine::ai::models::{Model, ModelInput};
use crate::engine::ai::providers::ModelProviders;
use crate::engine::ai::providers::processors::imagearray_to_ndarray::ImageArrayToNdArray;
use crate::engine::ai::providers::processors::{Preprocessor, PreprocessorData};
use crate::error::AIProxyError;
use ahnlich_types::ai::execution_provider::ExecutionProvider;
use ahnlich_types::ai::models::AiModel;
use ahnlich_types::ai::preprocess::PreprocessAction;
use ahnlich_types::keyval::StoreInput;
use ahnlich_types::keyval::store_input::Value;
use fallible_collections::FallibleVec;
use moka::future::Cache;
use ndarray::{Array, Ix4};
use rayon::prelude::*;
use task_manager::Task;
use task_manager::TaskManager;
use task_manager::TaskState;
use tokenizers::Encoding;
use tokio::sync::Mutex;
use tokio::sync::{mpsc, oneshot};
use tokio::time::Duration;
use tracing_opentelemetry::OpenTelemetrySpanExt;

type ModelThreadResponse = Result<Vec<ModelResponse>, AIProxyError>;

struct ModelThreadRequest {
    inputs: Arc<Vec<StoreInput>>,
    response: oneshot::Sender<ModelThreadResponse>,
    preprocess_action: PreprocessAction,
    action_type: InputAction,
    execution_provider: Option<ExecutionProvider>,
    model_params: std::collections::HashMap<String, String>,
    trace_span: tracing::Span,
}

struct ModelThread {
    model: Model,
    request_receiver: Mutex<mpsc::Receiver<ModelThreadRequest>>,
    enable_streaming: bool,
}

impl ModelThread {
    async fn new(
        supported_model: SupportedModels,
        cache_location: &Path,
        session_profiling: bool,
        enable_streaming: bool,
        request_receiver: mpsc::Receiver<ModelThreadRequest>,
    ) -> Result<Self, AIProxyError> {
        let model = supported_model
            .to_concrete_model(cache_location.to_path_buf(), session_profiling)
            .await?;
        Ok(Self {
            request_receiver: Mutex::new(request_receiver),
            model,
            enable_streaming,
        })
    }

    fn name(&self) -> String {
        format!("{:?}-model-thread", self.model.model_name())
    }

    #[tracing::instrument(skip(self, inputs))]
    async fn input_to_response(
        &self,
        inputs: Arc<Vec<StoreInput>>,
        process_action: PreprocessAction,
        action_type: InputAction,
        execution_provider: Option<ExecutionProvider>,
        model_params: &std::collections::HashMap<String, String>,
    ) -> ModelThreadResponse {
        let mut response: Vec<_> = FallibleVec::try_with_capacity(inputs.len())?;
        let processed_inputs = self.preprocess_store_input(process_action, inputs)?;
        let mut store_key = self
            .model
            .model_ndarray(
                processed_inputs,
                &action_type,
                execution_provider,
                model_params,
            )
            .await?;
        response.append(&mut store_key);
        Ok(response)
    }

    #[tracing::instrument(skip(self, inputs))]
    pub(crate) fn preprocess_store_input(
        &self,
        process_action: PreprocessAction,
        inputs: Arc<Vec<StoreInput>>,
    ) -> Result<ModelInput, AIProxyError> {
        let sample = inputs
            .first()
            .ok_or(AIProxyError::ModelPreprocessingError {
                model_name: self.model.model_name(),
                message: "Input is empty".to_string(),
            })?;
        let sample_type = sample
            .value
            .as_ref()
            .ok_or_else(|| AIProxyError::InputNotSpecified("Store input value".to_string()))?;
        match sample_type {
            Value::RawString(_) => {
                let inputs: Vec<String> = inputs
                    .par_iter()
                    .filter_map(|input| match &input.value {
                        Some(Value::RawString(string)) => Some(string.clone()),
                        _ => None,
                    })
                    .collect();
                let output = self.preprocess_raw_string(inputs, process_action)?;
                Ok(ModelInput::Texts(output))
            }
            Value::Image(_) => {
                if self.enable_streaming {
                    let batch_size = self.model.batch_size();
                    let output =
                        self.preprocess_images_chunked(inputs, process_action, batch_size)?;
                    Ok(ModelInput::Images(output))
                } else {
                    let image_arrays = inputs
                        .par_iter()
                        .filter_map(|input| match &input.value {
                            Some(Value::Image(image_bytes)) => {
                                Some(ImageArray::try_from(image_bytes.as_slice()).ok()?)
                            }
                            _ => None,
                        })
                        .collect();
                    let output = self.preprocess_image(image_arrays, process_action)?;
                    Ok(ModelInput::Images(output))
                }
            }
        }
    }
    #[tracing::instrument(skip(self, inputs))]
    fn preprocess_raw_string(
        &self,
        inputs: Vec<String>,
        process_action: PreprocessAction,
    ) -> Result<Vec<Encoding>, AIProxyError> {
        let max_token_size = usize::from(self.model.max_input_token().ok_or_else(|| {
            AIProxyError::ModelPreprocessingError {
                model_name: self.model.model_name(),
                message: "RawString preprocessing is not supported.".to_string(),
            }
        })?);

        match &self.model.provider {
            ModelProviders::ORT(provider) => {
                let truncate = matches!(process_action, PreprocessAction::ModelPreprocessing);
                let outputs = provider.preprocess_texts(inputs, truncate)?;
                let token_size = outputs
                    .first()
                    .ok_or(AIProxyError::ModelPreprocessingError {
                        model_name: self.model.model_name(),
                        message: "Processed output is empty".to_string(),
                    })?
                    .len();
                if token_size > max_token_size {
                    return Err(AIProxyError::TokenExceededError {
                        max_token_size,
                        input_token_size: token_size,
                    });
                } else {
                    return Ok(outputs);
                }
            }
        }
    }

    #[tracing::instrument(skip(self, inputs), fields(batch_size = batch_size, total_images = inputs.len()))]
    fn preprocess_images_chunked(
        &self,
        inputs: Arc<Vec<StoreInput>>,
        process_action: PreprocessAction,
        batch_size: usize,
    ) -> Result<Array<f32, Ix4>, AIProxyError> {
        let total_images = inputs.len();
        let mut all_preprocessed: Vec<Array<f32, Ix4>> =
            Vec::with_capacity(total_images.div_ceil(batch_size));

        // Process in chunks to limit peak memory
        for chunk_start in (0..total_images).step_by(batch_size) {
            let chunk_end = (chunk_start + batch_size).min(total_images);
            let chunk = &inputs[chunk_start..chunk_end];

            // Decode chunk
            let decoded_chunk: Vec<ImageArray> = chunk
                .par_iter()
                .filter_map(|input| match &input.value {
                    Some(Value::Image(image_bytes)) => {
                        Some(ImageArray::try_from(image_bytes.as_slice()).ok()?)
                    }
                    _ => None,
                })
                .collect();

            let preprocessed_chunk = self.preprocess_image(decoded_chunk, process_action)?;
            all_preprocessed.push(preprocessed_chunk);
            // decoded_chunk dropped here
        }

        if all_preprocessed.is_empty() {
            return Err(AIProxyError::ModelPreprocessingError {
                model_name: self.model.model_name(),
                message: "No images were successfully decoded".to_string(),
            });
        }

        // Concatenate chunks
        let views: Vec<_> = all_preprocessed.iter().map(|arr| arr.view()).collect();
        ndarray::concatenate(ndarray::Axis(0), &views).map_err(|e| {
            AIProxyError::ModelPreprocessingError {
                model_name: self.model.model_name(),
                message: format!("Failed to concatenate preprocessed chunks: {}", e),
            }
        })
    }

    #[tracing::instrument(skip(self, inputs))]
    fn preprocess_image(
        &self,
        inputs: Vec<ImageArray>,
        process_action: PreprocessAction,
    ) -> Result<Array<f32, Ix4>, AIProxyError> {
        // process image, return error if max dimensions exceeded
        let (expected_width, expected_height) = self.model.expected_image_dimensions().ok_or(
            AIProxyError::ModelPreprocessingError {
                model_name: self.model.model_name(),
                message: "Image preprocessing is not supported.".to_string(),
            },
        )?;
        let expected_width = usize::from(expected_width);
        let expected_height = usize::from(expected_height);

        match &self.model.provider {
            ModelProviders::ORT(provider) => {
                let outputs = match process_action {
                    PreprocessAction::ModelPreprocessing => provider.preprocess_images(inputs)?,
                    PreprocessAction::NoPreprocessing => ImageArrayToNdArray
                        .process(PreprocessorData::ImageArray(inputs))?
                        .into_ndarray3c()?,
                };
                let outputs_shape = outputs.shape();
                let width = *outputs_shape.get(2).expect("Must exist");
                let height = *outputs_shape.get(3).expect("Must exist");
                if width != expected_width || height != expected_height {
                    return Err(AIProxyError::ImageDimensionsMismatchError {
                        image_dimensions: (width, height),
                        expected_dimensions: (expected_width, expected_height),
                    });
                } else {
                    return Ok(outputs);
                }
            }
        }
    }
}

#[async_trait::async_trait]
impl Task for ModelThread {
    fn task_name(&self) -> String {
        self.name()
    }

    async fn run(&self) -> TaskState {
        let mut guard = self.request_receiver.lock().await;
        let request = guard.recv().await;
        drop(guard);
        if let Some(model_request) = request {
            let ModelThreadRequest {
                inputs,
                response,
                preprocess_action,
                action_type,
                execution_provider,
                model_params,
                trace_span,
            } = model_request;
            let child_span = tracing::info_span!("model-thread-run", model = self.task_name());
            child_span.set_parent(trace_span.context());

            let child_guard = child_span.enter();
            let responses = self
                .input_to_response(
                    inputs,
                    preprocess_action,
                    action_type,
                    execution_provider,
                    &model_params,
                )
                .await;
            if let Err(e) = response.send(responses) {
                log::error!("{} could not send response to channel {e:?}", self.name());
            }
            drop(child_guard);
            return TaskState::Continue;
        }
        TaskState::Break
    }
}

#[derive(Debug)]
pub struct ModelManager {
    models: Cache<SupportedModels, mpsc::Sender<ModelThreadRequest>>,
    supported_models: Vec<SupportedModels>,
    task_manager: OnceLock<Arc<TaskManager>>,
    config: ModelConfig,
}

impl ModelManager {
    pub fn new(model_config: ModelConfig) -> Self {
        let models = Cache::builder()
            .max_capacity(model_config.supported_models.len() as u64)
            .time_to_idle(Duration::from_secs(model_config.model_idle_time))
            .build();
        ModelManager {
            models,
            task_manager: OnceLock::new(),
            supported_models: model_config.supported_models.to_vec(),
            config: model_config,
        }
    }

    pub fn initialize_task_manager(
        &self,
        task_manager: Arc<TaskManager>,
    ) -> Result<(), Arc<TaskManager>> {
        self.task_manager.set(task_manager)
    }

    pub async fn spawn_model_threads(&self) -> Result<(), AIProxyError> {
        for model in &self.supported_models {
            let _ = self
                .models
                .try_get_with(*model, self.try_initialize_model(model))
                .await
                .map_err(|err| AIProxyError::ModelInitializationError(err.to_string()))?;
        }
        Ok(())
    }

    #[tracing::instrument(skip(self))]
    async fn try_initialize_model(
        &self,
        model: &SupportedModels,
    ) -> Result<mpsc::Sender<ModelThreadRequest>, AIProxyError> {
        let task_manager = self.task_manager.get().ok_or_else(|| {
            AIProxyError::ModelInitializationError(
                "task_manager must be initialized via spawn_model_threads".to_string(),
            )
        })?;
        let (request_sender, request_receiver) = mpsc::channel(10000);
        // There may be other things needed to load a model thread
        let model_thread = ModelThread::new(
            *model,
            &self.config.model_cache_location,
            self.config.session_profiling,
            self.config.enable_streaming,
            request_receiver,
        )
        .await?;
        let _ = task_manager.spawn_task_loop(model_thread).await;
        Ok(request_sender)
    }

    #[tracing::instrument(skip(self, inputs))]
    pub async fn handle_request(
        &self,
        model: &AiModel,
        inputs: Vec<StoreInput>,
        preprocess_action: PreprocessAction,
        action_type: InputAction,
        execution_provider: Option<ExecutionProvider>,
        model_params: std::collections::HashMap<String, String>,
    ) -> Result<Vec<ModelResponse>, AIProxyError> {
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
            inputs: Arc::new(inputs),
            response: response_tx,
            preprocess_action,
            action_type,
            execution_provider,
            model_params,
            trace_span: tracing::Span::current(),
        };
        // TODO: Add potential timeouts for send and recieve in case threads are unresponsive
        sender
            .send(request)
            .await
            .map_err(|a| AIProxyError::AIModelThreadSendError {
                message: a.to_string(),
            })?;
        response_rx
            .await
            .map_err(|e| e.into())
            .and_then(|inner| inner)
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
        let model_manager = ModelManager::new(model_config);
        model_manager
            .initialize_task_manager(task_manager)
            .expect("task_manager should not be set yet");
        model_manager.spawn_model_threads().await.unwrap();

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
        let sample_ai_model = AiModel::AllMiniLmL6V2;
        let sample_supported_model: SupportedModels = (&sample_ai_model).into();

        let task_manager = Arc::new(TaskManager::new());
        let time_to_idle: u64 = 1;
        let model_config = ModelConfig {
            supported_models: supported_models.clone(),
            model_idle_time: time_to_idle,
            ..Default::default()
        };
        let model_manager = ModelManager::new(model_config);
        model_manager
            .initialize_task_manager(task_manager)
            .expect("task_manager should not be set yet");
        model_manager.spawn_model_threads().await.unwrap();
        let _ = tokio::time::sleep(Duration::from_secs(2)).await;

        let evicted_model = model_manager.models.get(&sample_supported_model).await;

        let inputs = vec![StoreInput {
            value: Some(Value::RawString(String::from("Hello"))),
        }];
        let action = PreprocessAction::ModelPreprocessing;
        let _ = model_manager
            .handle_request(
                &sample_ai_model,
                inputs,
                action,
                InputAction::Query,
                None,
                std::collections::HashMap::new(),
            )
            .await
            .unwrap();
        let recreated_model = model_manager.models.get(&sample_supported_model).await;

        assert!(evicted_model.is_none());
        assert!(recreated_model.is_some());
    }
}
