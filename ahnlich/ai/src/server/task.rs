use crate::engine::ai::models::Model;
use ahnlich_client_rs::db::DbClient;
use ahnlich_types::ai::{
    AIQuery, AIServerQuery, AIServerResponse, AIServerResult, ImageAction, PreprocessAction,
    StringAction,
};
use ahnlich_types::client::ConnectedClient;
use ahnlich_types::db::{ServerInfo, ServerResponse};
use ahnlich_types::keyval::{StoreInput, StoreValue};
use ahnlich_types::metadata::MetadataValue;
use ahnlich_types::predicate::{Predicate, PredicateCondition};
use ahnlich_types::version::VERSION;
use fallible_collections::vec::TryFromIterator;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use task_manager::Task;
use task_manager::TaskState;
use tokio::io::BufReader;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tracing::Instrument;
use utils::allocator::GLOBAL_ALLOCATOR;
use utils::client::ClientHandler;
use utils::protocol::AhnlichProtocol;

use crate::engine::store::AIStoreHandler;
use crate::error::AIProxyError;
use crate::manager::ModelManager;
use crate::AHNLICH_AI_RESERVED_META_KEY;

#[derive(Debug)]
pub struct AIProxyTask {
    pub(super) server_addr: SocketAddr,
    pub(super) reader: Arc<Mutex<BufReader<TcpStream>>>,
    pub(super) client_handler: Arc<ClientHandler>,
    pub(super) store_handler: Arc<AIStoreHandler>,
    pub(super) connected_client: ConnectedClient,
    pub(super) maximum_message_size: u64,
    pub(super) db_client: Arc<DbClient>,
    pub(super) model_manager: Arc<ModelManager>,
}

#[async_trait::async_trait]
impl AhnlichProtocol for AIProxyTask {
    type ServerQuery = AIServerQuery;
    type ServerResponse = AIServerResult;

    fn connected_client(&self) -> &ConnectedClient {
        &self.connected_client
    }
    fn maximum_message_size(&self) -> u64 {
        self.maximum_message_size
    }
    fn reader(&self) -> Arc<Mutex<BufReader<TcpStream>>> {
        self.reader.clone()
    }

    async fn handle(&self, queries: Vec<AIQuery>) -> AIServerResult {
        let mut result = AIServerResult::with_capacity(queries.len());
        let parent_id = tracer::span_to_trace_parent(tracing::Span::current());
        for query in queries {
            result.push(match query {
                AIQuery::Ping => Ok(AIServerResponse::Pong),
                AIQuery::ListStores => Ok(AIServerResponse::StoreList(
                    self.store_handler.list_stores(),
                )),
                AIQuery::InfoServer => Ok(AIServerResponse::InfoServer(self.server_info())),

                AIQuery::CreateStore {
                    store,
                    query_model,
                    index_model,
                    mut predicates,
                    non_linear_indices,
                    error_if_exists,
                    store_original,
                } => {
                    let default_metadata_key = &*AHNLICH_AI_RESERVED_META_KEY;
                    if predicates.contains(&AHNLICH_AI_RESERVED_META_KEY) {
                        Err(format!("Cannot use {} keyword", default_metadata_key))
                    } else {
                        predicates.insert(default_metadata_key.clone());
                        let model: Model = (&index_model).into();
                        match self
                            .db_client
                            .create_store(
                                store.clone(),
                                model.embedding_size,
                                predicates,
                                non_linear_indices,
                                false,
                                parent_id.clone(),
                            )
                            .await
                        {
                            Err(err) => Err(err.to_string()),
                            Ok(_) => self
                                .store_handler
                                .create_store(
                                    store,
                                    query_model,
                                    index_model,
                                    error_if_exists,
                                    store_original,
                                )
                                .map(|_| AIServerResponse::Unit)
                                .map_err(|e| e.to_string()),
                        }
                    }
                }

                AIQuery::Set {
                    store,
                    inputs,
                    preprocess_action,
                } => {
                    let model_manager = &self.model_manager;
                    let default_metadatakey = &*AHNLICH_AI_RESERVED_META_KEY;

                    match self
                        .store_handler
                        .set(&store, inputs, model_manager, preprocess_action)
                        .await
                    {
                        Ok((db_inputs, delete_hashset)) => {
                            match self.db_client.pipeline(2, parent_id.clone()).await {
                                Err(err) => Err(err.to_string()),
                                Ok(mut pipeline) => {
                                    if let Some(del_hashset) = delete_hashset {
                                        let delete_condition =
                                            PredicateCondition::Value(Predicate::In {
                                                key: default_metadatakey.clone(),
                                                value: del_hashset,
                                            });
                                        pipeline.del_pred(store.clone(), delete_condition);
                                    }
                                    pipeline.set(store, db_inputs);
                                    match pipeline.exec().await {
                                        Ok(res) => match res.into_inner().as_slice() {
                                            [Ok(ServerResponse::Set(upsert))]
                                            | [Ok(_), Ok(ServerResponse::Set(upsert))] => {
                                                Ok(AIServerResponse::Set(upsert.clone()))
                                            }
                                            e => Err(AIProxyError::UnexpectedDBResponse(format!(
                                                "{e:?}"
                                            ))
                                            .to_string()),
                                        },
                                        Err(err) => Err(format!("{err}")),
                                    }
                                }
                            }
                        }
                        Err(err) => Err(err.to_string()),
                    }
                }

                AIQuery::DelKey { store, key } => {
                    let default_metadatakey = &*AHNLICH_AI_RESERVED_META_KEY;
                    let metadata_value: MetadataValue = key.into();
                    let delete_condition = PredicateCondition::Value(Predicate::In {
                        key: default_metadatakey.clone(),
                        value: HashSet::from_iter([metadata_value]),
                    });

                    match self
                        .db_client
                        .del_pred(store, delete_condition, parent_id.clone())
                        .await
                    {
                        Ok(res) => {
                            if let ServerResponse::Del(num) = res {
                                Ok(AIServerResponse::Del(num))
                            } else {
                                Err(AIProxyError::UnexpectedDBResponse(format!("{:?}", res))
                                    .to_string())
                            }
                        }
                        Err(err) => Err(format!("{err}")),
                    }
                }
                AIQuery::DropStore {
                    store,
                    error_if_not_exists,
                } => match self
                    .db_client
                    .drop_store(store.clone(), error_if_not_exists, parent_id.clone())
                    .await
                {
                    Ok(_) => self
                        .store_handler
                        .drop_store(store, error_if_not_exists)
                        .map(AIServerResponse::Del)
                        .map_err(|e| e.to_string()),
                    Err(err) => Err(format!("{err}")),
                },
                AIQuery::CreatePredIndex { store, predicates } => {
                    if predicates.contains(&*AHNLICH_AI_RESERVED_META_KEY) {
                        Err(format!(
                            "Cannot use {} keyword",
                            *AHNLICH_AI_RESERVED_META_KEY
                        ))
                    } else {
                        match self
                            .db_client
                            .create_pred_index(store, predicates, parent_id.clone())
                            .await
                        {
                            Ok(res) => {
                                if let ServerResponse::CreateIndex(num) = res {
                                    Ok(AIServerResponse::CreateIndex(num))
                                } else {
                                    Err(AIProxyError::UnexpectedDBResponse(format!("{:?}", res))
                                        .to_string())
                                }
                            }
                            Err(err) => Err(format!("{err}")),
                        }
                    }
                }
                AIQuery::CreateNonLinearAlgorithmIndex {
                    store,
                    non_linear_indices,
                } => {
                    match self
                        .db_client
                        .create_non_linear_algorithm_index(
                            store,
                            non_linear_indices,
                            parent_id.clone(),
                        )
                        .await
                    {
                        Ok(res) => {
                            if let ServerResponse::CreateIndex(num) = res {
                                Ok(AIServerResponse::CreateIndex(num))
                            } else {
                                Err(AIProxyError::UnexpectedDBResponse(format!("{:?}", res))
                                    .to_string())
                            }
                        }
                        Err(err) => Err(format!("{err}")),
                    }
                }
                AIQuery::DropPredIndex {
                    store,
                    mut predicates,
                    error_if_not_exists,
                } => {
                    let default_metadatakey = &*AHNLICH_AI_RESERVED_META_KEY;
                    if predicates.contains(default_metadatakey) {
                        let _ = predicates.remove(default_metadatakey);
                    }
                    match self
                        .db_client
                        .drop_pred_index(store, predicates, error_if_not_exists, parent_id.clone())
                        .await
                    {
                        Ok(res) => {
                            if let ServerResponse::Del(num) = res {
                                Ok(AIServerResponse::Del(num))
                            } else {
                                Err(AIProxyError::UnexpectedDBResponse(format!("{:?}", res))
                                    .to_string())
                            }
                        }
                        Err(err) => Err(format!("{err}")),
                    }
                }
                AIQuery::DropNonLinearAlgorithmIndex {
                    store,
                    non_linear_indices,
                    error_if_not_exists,
                } => {
                    match self
                        .db_client
                        .drop_non_linear_algorithm_index(
                            store,
                            non_linear_indices,
                            error_if_not_exists,
                            parent_id.clone(),
                        )
                        .await
                    {
                        Ok(res) => {
                            if let ServerResponse::Del(num) = res {
                                Ok(AIServerResponse::Del(num))
                            } else {
                                Err(AIProxyError::UnexpectedDBResponse(format!("{:?}", res))
                                    .to_string())
                            }
                        }
                        Err(err) => Err(format!("{err}")),
                    }
                }
                AIQuery::GetPred { store, condition } => {
                    match self
                        .db_client
                        .get_pred(store, condition, parent_id.clone())
                        .await
                    {
                        Ok(res) => {
                            if let ServerResponse::Get(response) = res {
                                // conversion to store input here
                                let output: Vec<(StoreInput, StoreValue)> = self
                                    .store_handler
                                    .store_key_val_to_store_input_val(response);
                                Ok(AIServerResponse::Get(output))
                            } else {
                                Err(AIProxyError::UnexpectedDBResponse(format!("{:?}", res))
                                    .to_string())
                            }
                        }
                        Err(err) => Err(format!("{err}")),
                    }
                }
                AIQuery::GetSimN {
                    store,
                    search_input,
                    condition,
                    closest_n,
                    algorithm,
                } => {
                    // TODO: Replace this with calls to self.model_manager.handle_request
                    // TODO (HAKSOAT): Shouldn't preprocess action also be in the params?
                    let preprocess = match search_input {
                        StoreInput::RawString(_) => {
                            PreprocessAction::RawString(StringAction::TruncateIfTokensExceed)
                        }
                        StoreInput::Image(_) => PreprocessAction::Image(ImageAction::ResizeImage),
                    };
                    let repr = self
                        .store_handler
                        .get_ndarray_repr_for_store(
                            &store,
                            search_input,
                            &self.model_manager,
                            preprocess,
                        )
                        .await;
                    if let Ok(store_key) = repr {
                        match self
                            .db_client
                            .get_sim_n(
                                store,
                                store_key,
                                closest_n,
                                algorithm,
                                condition,
                                parent_id.clone(),
                            )
                            .await
                        {
                            Ok(res) => {
                                if let ServerResponse::GetSimN(response) = res {
                                    TryFromIterator::try_from_iterator(
                                        response.into_iter().flat_map(
                                            |(store_key, store_value, sim)| {
                                                // TODO: Can parallelize
                                                self.store_handler
                                                    .store_key_val_to_store_input_val(vec![(
                                                        store_key,
                                                        store_value,
                                                    )])
                                                    .into_iter()
                                                    .map(move |v| (v.0, v.1, sim))
                                            },
                                        ),
                                    )
                                    .map_err(|e| AIProxyError::from(e).to_string())
                                    .map(AIServerResponse::GetSimN)
                                } else {
                                    Err(AIProxyError::UnexpectedDBResponse(format!("{:?}", res))
                                        .to_string())
                                }
                            }
                            Err(err) => Err(format!("{err}")),
                        }
                    } else {
                        Err(
                            AIProxyError::StandardError("Failed to get store".to_string())
                                .to_string(),
                        )
                    }
                }
                AIQuery::PurgeStores => {
                    let destoryed = self.store_handler.purge_stores();
                    Ok(AIServerResponse::Del(destoryed))
                }
            })
        }
        result
    }
}

impl AIProxyTask {
    #[tracing::instrument(skip(self))]
    fn server_info(&self) -> ServerInfo {
        ServerInfo {
            address: format!("{}", self.server_addr),
            version: *VERSION,
            r#type: ahnlich_types::ServerType::AI,
            limit: GLOBAL_ALLOCATOR.limit(),
            remaining: GLOBAL_ALLOCATOR.remaining(),
        }
    }
}

#[async_trait::async_trait]
impl Task for AIProxyTask {
    fn task_name(&self) -> String {
        format!("ai-{}-connection", self.connected_client.address)
    }

    async fn run(&self) -> TaskState {
        self.process()
            .instrument(tracing::info_span!("ai-server-listener"))
            .await
    }
}

impl Drop for AIProxyTask {
    fn drop(&mut self) {
        self.client_handler.disconnect(&self.connected_client);
    }
}
