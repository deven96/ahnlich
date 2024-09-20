use super::ai::models::Model;
use crate::{error::AIProxyError, AHNLICH_AI_RESERVED_META_KEY};
use ahnlich_types::{
    ai::{AIModel, AIStoreInputType},
    keyval::{StoreInput, StoreValue},
    metadata::MetadataValue,
};
use fallible_collections::FallibleVec;
use std::{
    collections::{HashMap as StdHashMap, HashSet as StdHashSet},
    num::NonZeroUsize,
    sync::Arc,
};
use task_manager::{Task, TaskManager, TaskState};
use tokio::sync::{mpsc, oneshot, Mutex};

type StoreValidateResponse =
    Result<(Vec<(StoreInput, StoreValue)>, StdHashSet<MetadataValue>), AIProxyError>;
type Job = Vec<(StoreInput, StoreValue)>;

pub(crate) struct StoreValidationRequest {
    tasks: Vec<(StoreInput, StoreValue)>,
    response: oneshot::Sender<StoreValidateResponse>,
    index_model: AIModel,
}

#[async_trait::async_trait]
impl Task for Worker {
    fn task_name(&self) -> String {
        self.name()
    }

    #[tracing::instrument(skip(self))]
    async fn run(&self) -> TaskState {
        if let Some(validation_request) = self.request_receiver.lock().await.recv().await {
            // TODO actually service model request in here and return
            let StoreValidationRequest {
                tasks,
                response,
                index_model,
            } = validation_request;
            //let current_span = tracing::Span::current();
            //current_span.set_parent(trace_span.context());
            if let Err(e) = response.send(self.process_store_inputs(index_model, tasks).await) {
                log::error!("{} could not send response to channel {e:?}", self.name());
            }
            return TaskState::Continue;
        }
        TaskState::Break
    }
}

#[derive(Debug)]
pub(crate) struct WorkerName;

impl WorkerName {
    pub(crate) fn name(idx: usize) -> String {
        format!("Worker-{idx}")
    }
}

pub(crate) struct Worker {
    id: usize,
    request_receiver: Arc<Mutex<mpsc::Receiver<StoreValidationRequest>>>,
}
impl Worker {
    pub(crate) fn new(
        id: usize,
        request_receiver: Arc<Mutex<mpsc::Receiver<StoreValidationRequest>>>,
    ) -> Self {
        Self {
            id,
            request_receiver,
        }
    }

    fn name(&self) -> String {
        WorkerName::name(self.id)
    }

    #[tracing::instrument(skip(self, inputs))]
    pub(crate) async fn process_store_inputs(
        &self,
        index_model: AIModel,
        inputs: Vec<(StoreInput, StoreValue)>,
    ) -> StoreValidateResponse {
        let mut output: Vec<_> = FallibleVec::try_with_capacity(inputs.len())?;
        let mut delete_hashset = StdHashSet::new();
        let metadata_key = &*AHNLICH_AI_RESERVED_META_KEY;
        for (store_input, mut store_value) in inputs.into_iter() {
            if store_value.contains_key(metadata_key) {
                return Err(AIProxyError::ReservedError(metadata_key.to_string()));
            }
            let store_input_type: AIStoreInputType = (&store_input).into();
            let index_model_repr: Model = (&index_model).into();
            if store_input_type.to_string() != index_model_repr.input_type() {
                return Err(AIProxyError::StoreSetTypeMismatchError {
                    index_model_type: index_model_repr.input_type(),
                    storeinput_type: store_input_type.to_string(),
                });
            }
            let metadata_value: MetadataValue = store_input.clone().into();
            store_value.insert(metadata_key.clone(), metadata_value.clone());
            output.try_push((store_input, store_value))?;
            delete_hashset.insert(metadata_value);
        }

        Ok((output, delete_hashset))
    }
}

pub(crate) struct StoreValidatorThreadPool {
    pool_size: NonZeroUsize,
    handles: StdHashMap<String, mpsc::Sender<StoreValidationRequest>>,
}

impl StoreValidatorThreadPool {
    pub(crate) async fn build(
        pool_size: NonZeroUsize,
        task_manager: &TaskManager,
    ) -> Result<Self, AIProxyError> {
        let mut handles = StdHashMap::new();
        for id in 0..pool_size.into() {
            let (sender, receiver) = mpsc::channel(pool_size.into());
            let receiver = Arc::new(Mutex::new(receiver));
            let worker = Worker::new(id, Arc::clone(&receiver));
            handles.insert(worker.name(), sender);
            let _ = task_manager.spawn_task_loop(worker).await;
        }
        Ok(Self { handles, pool_size })
    }

    pub(crate) async fn execute(&self, inputs: Job, index_model: AIModel) -> StoreValidateResponse {
        let mut output: Vec<_> = FallibleVec::try_with_capacity(inputs.len())?;
        let mut delete_hashset = StdHashSet::new();

        let chunked = inputs.chunks(self.pool_size.into());

        for (idx, chunk) in chunked.into_iter().enumerate() {
            let (response_tx, response_rx) = oneshot::channel();
            let request = StoreValidationRequest {
                tasks: chunk.to_vec(),
                response: response_tx,
                index_model,
            };
            // TODO: Add potential timeouts for send and recieve in case threads are unresponsive
            if let Some(sender) = self.handles.get(&WorkerName::name(idx)) {
                if sender.send(request).await.is_ok() {
                    let response = response_rx
                        .await
                        .map_err(|e| e.into())
                        .and_then(|inner| inner);
                    match response {
                        Ok(values) => {
                            let (temp_output, temp_mv) = values;
                            output.extend(temp_output); //exted(temp_output);
                            delete_hashset.extend(temp_mv); //.insert(temp_mv);
                        }
                        Err(err) => return Err(err),
                    }
                } else {
                    return Err(AIProxyError::AIModelThreadSendError);
                };
            }
        }
        Ok((output, delete_hashset))
    }
}
