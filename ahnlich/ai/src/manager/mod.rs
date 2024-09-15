use std::sync::Arc;
use std::time::Duration;

/// The ModelManager is a wrapper around all the AI models running on various green threads. It
/// lets AIProxyTasks communicate with any model to receive immediate responses via a oneshot
/// channel
use crate::cli::server::SupportedModels;
use crate::engine::ai::models::Model;
use crate::error::AIProxyError;
use ahnlich_types::ai::{AIModel, AIStoreInputType, ImageAction, PreprocessAction, StringAction};
use ahnlich_types::keyval::{StoreInput, StoreKey};
use fallible_collections::FallibleVec;
use moka::future::Cache;
use task_manager::Task;
use task_manager::TaskManager;
use task_manager::TaskState;
use tokio::sync::Mutex;
use tokio::sync::{mpsc, oneshot};

type ModelThreadResponse = Result<Vec<StoreKey>, AIProxyError>;

struct ModelThreadRequest {
    // TODO: change this to a Vec of preprocessed input enum
    inputs: Vec<StoreInput>,
    response: oneshot::Sender<ModelThreadResponse>,
    preprocess_action: PreprocessAction,
}

#[derive(Debug)]
struct ModelThread {
    model: SupportedModels,
    request_receiver: Mutex<mpsc::Receiver<ModelThreadRequest>>,
}

impl ModelThread {
    fn new(model: SupportedModels, request_receiver: mpsc::Receiver<ModelThreadRequest>) -> Self {
        Self {
            request_receiver: Mutex::new(request_receiver),
            model,
        }
    }

    fn name(&self) -> String {
        format!("{:?}-model-thread", self.model)
    }

    fn input_to_response(
        &self,
        inputs: Vec<StoreInput>,
        process_action: PreprocessAction,
    ) -> ModelThreadResponse {
        let model: Model = (&self.model).into();
        let mut response: Vec<_> = FallibleVec::try_with_capacity(inputs.len())?;
        // move this from for loop into vec of inputs
        for input in inputs {
            let processed_input = self.preprocess_store_input(process_action, input)?;
            let store_key = model.model_ndarray(&processed_input);
            response.push(store_key);
        }
        Ok(response)
    }

    #[tracing::instrument(skip(self))]
    pub(crate) fn preprocess_store_input(
        &self,
        process_action: PreprocessAction,
        input: StoreInput,
    ) -> Result<StoreInput, AIProxyError> {
        match (process_action, input) {
            (PreprocessAction::Image(image_action), StoreInput::Image(image_input)) => {
                // resize image and edit
                let output = self.process_image(image_input, image_action)?;
                Ok(output)
            }
            (PreprocessAction::RawString(string_action), StoreInput::RawString(string_input)) => {
                let output = self.preprocess_raw_string(string_input, string_action)?;
                Ok(output)
            }
            (PreprocessAction::RawString(_), StoreInput::Image(_)) => {
                Err(AIProxyError::PreprocessingMismatchError {
                    input_type: AIStoreInputType::Image,
                    preprocess_action: process_action,
                })
            }

            (PreprocessAction::Image(_), StoreInput::RawString(_)) => {
                Err(AIProxyError::PreprocessingMismatchError {
                    input_type: AIStoreInputType::RawString,
                    preprocess_action: process_action,
                })
            }
        }
    }
    #[tracing::instrument(skip(self))]
    fn preprocess_raw_string(
        &self,
        input: String,
        string_action: StringAction,
    ) -> Result<StoreInput, AIProxyError> {
        // tokenize string, return error if max token
        //let tokenized_input;
        let model: Model = (&self.model).into();

        if let Some(max_token_size) = model.max_input_token() {
            if input.len() > max_token_size.into() {
                if let StringAction::ErrorIfTokensExceed = string_action {
                    return Err(AIProxyError::TokenExceededError {
                        input_token_size: input.len(),
                        max_token_size: max_token_size.into(),
                    });
                } else {
                    // truncate raw string
                    // let tokenized_input;
                    let _input = input.as_str()[..max_token_size.into()].to_string();
                }
            }
        }
        Ok(StoreInput::RawString(input))
    }

    #[tracing::instrument(skip(self))]
    fn process_image(
        &self,
        input: Vec<u8>,
        image_action: ImageAction,
    ) -> Result<StoreInput, AIProxyError> {
        // process image, return error if max dimensions exceeded
        // let image_data;
        let model: Model = (&self.model).into();

        if let Some(max_image_dimensions) = model.max_image_dimensions() {
            if input.len() > max_image_dimensions.into() {
                if let ImageAction::ErrorIfDimensionsMismatch = image_action {
                    return Err(AIProxyError::ImageDimensionsMismatchError {
                        image_dimensions: input.len(),
                        max_dimensions: max_image_dimensions.into(),
                    });
                } else {
                    // resize image
                }
            }
        }
        Ok(StoreInput::Image(input))
    }
}

#[async_trait::async_trait]
impl Task for ModelThread {
    fn task_name(&self) -> String {
        self.name()
    }

    #[tracing::instrument(skip(self))]
    async fn run(&self) -> TaskState {
        if let Some(model_request) = self.request_receiver.lock().await.recv().await {
            // TODO actually service model request in here and return
            let ModelThreadRequest {
                inputs,
                response,
                preprocess_action,
            } = model_request;
            if let Err(e) = response.send(self.input_to_response(inputs, preprocess_action)) {
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
}

impl ModelManager {
    pub async fn new(
        supported_models: &[SupportedModels],
        task_manager: Arc<TaskManager>,
    ) -> Result<Self, AIProxyError> {
        // TODO: Actually load the model at this point, with supported models and throw up errors,
        // also start up the various models with the task manager passed in
        //
        let tti = Duration::from_secs(10);
        let models = Cache::builder()
            .max_capacity(supported_models.len() as u64)
            .time_to_idle(tti)
            .build();
        for model in supported_models {
            let request_sender = Self::create_request_sender(model, &task_manager).await?;
            models.insert(*model, request_sender).await;
        }
        Ok(Self {
            models,
            supported_models: supported_models.to_vec(),
            task_manager,
        })
    }

    // QUESTION? Should we hard code the buffer size or add it to config?
    async fn create_request_sender(
        supported_model: &SupportedModels,
        task_manager: &TaskManager,
    ) -> Result<mpsc::Sender<ModelThreadRequest>, AIProxyError> {
        let (request_sender, request_receiver) = mpsc::channel(100);
        // There may be other things needed to load a model thread
        let model_thread = ModelThread::new(*supported_model, request_receiver);
        task_manager.spawn_task_loop(model_thread).await;
        Ok(request_sender)
    }

    async fn get_sender(
        &self,
        model: &SupportedModels,
    ) -> Result<mpsc::Sender<ModelThreadRequest>, AIProxyError> {
        let request_sender = Self::create_request_sender(model, &self.task_manager).await?;
        self.models.insert(*model, request_sender.clone()).await;
        Ok(request_sender)
    }

    #[tracing::instrument(skip(self, inputs))]
    pub async fn handle_request(
        &self,
        model: &AIModel,
        inputs: Vec<StoreInput>,
        preprocess_action: PreprocessAction,
    ) -> Result<Vec<StoreKey>, AIProxyError> {
        let supported = model.into();

        if !self.supported_models.contains(&supported) {
            return Err(AIProxyError::AIModelNotInitialized);
        }

        let sender = self
            .models
            .try_get_with(supported, self.get_sender(&supported))
            .await
            .map_err(|err| AIProxyError::StandardError(err.to_string()))?;

        let (response_tx, response_rx) = oneshot::channel();
        let request = ModelThreadRequest {
            inputs,
            response: response_tx,
            preprocess_action,
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
