/// The ModelManager is a wrapper around all the AI models running on various green threads. It
/// lets AIProxyTasks communicate with any model to receive immediate responses via a oneshot
/// channel
use crate::cli::server::SupportedModels;
use crate::error::AIProxyError;
use ahnlich_types::ai::AIModel;
use ndarray::Array1;
use std::collections::HashMap;
use task_manager::TaskManager;
use task_manager::TaskManagerGuard;
use tokio::sync::{mpsc, oneshot};

type ModelThreadResponse = Result<Vec<Array1<f32>>, AIProxyError>;

struct ModelThreadRequest {
    // TODO: change this to a Vec of preprocessed input enum
    inputs: (),
    response: oneshot::Sender<ModelThreadResponse>,
}

#[derive(Debug)]
struct ModelThread {
    model: SupportedModels,
    request_receiver: mpsc::Receiver<ModelThreadRequest>,
}

impl ModelThread {
    fn new(model: SupportedModels, request_receiver: mpsc::Receiver<ModelThreadRequest>) -> Self {
        Self {
            request_receiver,
            model,
        }
    }

    fn name(&self) -> String {
        format!("{:?}-model-thread", self.model)
    }

    fn input_to_response(&self, _inputs: ()) -> ModelThreadResponse {
        Ok(vec![])
    }

    #[tracing::instrument(skip(self))]
    async fn run(&mut self, guard: TaskManagerGuard) {
        loop {
            tokio::select! {
                biased;

                _ = guard.is_cancelled() => {
                    // TODO: Run cleanup for model thread exit
                    // This should be safe to do as the guard would wait
                    break;
                }
                Some(model_request) = self.request_receiver.recv() => {
                    // TODO actually service model request in here and return
                    let ModelThreadRequest {
                        inputs,
                        response,
                    } = model_request;
                    if let Err(e) = response.send(self.input_to_response(inputs)) {
                        log::error!("{} could not send response to channel {e:?}", self.name());
                    }
                }
            }
        }
    }
}

#[derive(Debug)]
pub struct ModelManager {
    _models: HashMap<SupportedModels, mpsc::Sender<ModelThreadRequest>>,
}

impl ModelManager {
    pub async fn new(
        supported_models: &[SupportedModels],
        task_manager: &TaskManager,
    ) -> Result<Self, AIProxyError> {
        // TODO: Actually load the model at this point, with supported models and throw up errors,
        // also start up the various models with the task manager passed in
        //
        let mut models = HashMap::with_capacity(supported_models.len());
        for model in supported_models {
            // QUESTION? Should we hard code the buffer size or add it to config?
            let (request_sender, request_receiver) = mpsc::channel(100);
            // There may be other things needed to load a model thread
            let mut model_thread = ModelThread::new(*model, request_receiver);
            let model_thread_name = model_thread.name();
            task_manager
                .spawn_task_loop(
                    |shutdown_guard| async move { model_thread.run(shutdown_guard).await },
                    model_thread_name,
                )
                .await;
            models.insert(model, request_sender);
        }
        Ok(Self {
            _models: HashMap::new(),
        })
    }

    #[tracing::instrument(skip(self, inputs))]
    pub async fn handle_request(
        &self,
        model: &AIModel,
        inputs: (),
    ) -> Result<Vec<Array1<f32>>, AIProxyError> {
        let supported = model.into();
        if let Some(sender) = self._models.get(&supported) {
            let (response_tx, response_rx) = oneshot::channel();
            let request = ModelThreadRequest {
                inputs,
                response: response_tx,
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
        } else {
            return Err(AIProxyError::AIModelNotInitialized);
        }
    }
}
