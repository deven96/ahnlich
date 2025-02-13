use crate::cli::server::SupportedModels;
use crate::engine::ai::models::ModelDetails;
use ahnlich_client_rs::{builders::db as db_params, db::DbClient};
use ahnlich_types::ai::{AIQuery, AIServerQuery, AIServerResponse, AIServerResult};
use ahnlich_types::client::ConnectedClient;
use ahnlich_types::db::{ServerInfo, ServerResponse};
use ahnlich_types::metadata::MetadataValue;
use ahnlich_types::predicate::{Predicate, PredicateCondition};
use ahnlich_types::version::VERSION;
use rayon::prelude::*;
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
                    if store_original {
                        predicates.insert(default_metadata_key.clone());
                    }
                    let model: ModelDetails =
                        SupportedModels::from(&index_model).to_model_details();
                    let create_store_params = db_params::CreateStoreParams::builder()
                        .store(store.clone().to_string())
                        .dimension(model.embedding_size.into())
                        .create_predicates(predicates)
                        .non_linear_indices(non_linear_indices)
                        .error_if_exists(false)
                        .tracing_id(parent_id.clone())
                        .build();
                    match self.db_client.create_store(create_store_params).await {
                        Err(err) => {
                            tracing::error!("{:?}", err);
                            Err(err.to_string())
                        }
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
                            .map_err(|e| {
                                tracing::error!("{e}");
                                e.to_string()
                            }),
                    }
                }

                AIQuery::Set {
                    store,
                    inputs,
                    preprocess_action,
                    execution_provider,
                } => {
                    let model_manager = &self.model_manager;

                    match self
                        .store_handler
                        .set(
                            &store,
                            inputs,
                            model_manager,
                            preprocess_action,
                            execution_provider,
                        )
                        .await
                    {
                        Ok((db_inputs, delete_hashset)) => {
                            match self.db_client.pipeline(2, parent_id.clone()).await {
                                Err(err) => {
                                    tracing::error!("{:?}", err);
                                    Err(err.to_string())
                                }
                                Ok(mut pipeline) => {
                                    if let Some(del_hashset) = delete_hashset {
                                        let default_metadatakey = &*AHNLICH_AI_RESERVED_META_KEY;
                                        let delete_condition =
                                            PredicateCondition::Value(Predicate::In {
                                                key: default_metadatakey.clone(),
                                                value: del_hashset,
                                            });
                                        let del_pred_params = db_params::DelPredParams::builder()
                                            .store(store.clone().to_string())
                                            .condition(delete_condition)
                                            .tracing_id(parent_id.clone())
                                            .build();
                                        pipeline.del_pred(del_pred_params);
                                    }
                                    let set_params = db_params::SetParams::builder()
                                        .store(store.to_string())
                                        .inputs(db_inputs)
                                        .tracing_id(parent_id.clone())
                                        .build();

                                    pipeline.set(set_params);
                                    match pipeline.exec().await {
                                        Ok(res) => match res.into_inner().as_slice() {
                                            [Ok(ServerResponse::Set(upsert))]
                                            | [Ok(_), Ok(ServerResponse::Set(upsert))] => {
                                                Ok(AIServerResponse::Set(upsert.clone()))
                                            }
                                            e => {
                                                tracing::error!("{:?}", e);
                                                Err(AIProxyError::UnexpectedDBResponse(format!(
                                                    "{e:?}"
                                                ))
                                                .to_string())
                                            }
                                        },
                                        Err(err) => {
                                            tracing::error!("{:?}", err);
                                            Err(format!("{err}"))
                                        }
                                    }
                                }
                            }
                        }
                        Err(err) => {
                            tracing::error!("{err}");
                            Err(err.to_string())
                        }
                    }
                }

                AIQuery::DelKey { store, key } => {
                    match self.store_handler.store_original(store.clone()) {
                        Err(err) => {
                            tracing::error!("{:?}", err);
                            Err(err.to_string())
                        }
                        Ok(false) => Err(AIProxyError::DelKeyError.to_string()),
                        Ok(true) => {
                            let default_metadatakey = &*AHNLICH_AI_RESERVED_META_KEY;
                            let metadata_value: MetadataValue = key.into();
                            let delete_condition = PredicateCondition::Value(Predicate::In {
                                key: default_metadatakey.clone(),
                                value: HashSet::from_iter([metadata_value]),
                            });
                            let del_pred_params = db_params::DelPredParams::builder()
                                .store(store.to_string())
                                .condition(delete_condition)
                                .tracing_id(parent_id.clone())
                                .build();
                            match self.db_client.del_pred(del_pred_params).await {
                                Ok(res) => {
                                    if let ServerResponse::Del(num) = res {
                                        Ok(AIServerResponse::Del(num))
                                    } else {
                                        tracing::error!("{:?}", res);
                                        Err(AIProxyError::UnexpectedDBResponse(format!(
                                            "{:?}",
                                            res
                                        ))
                                        .to_string())
                                    }
                                }
                                Err(err) => {
                                    tracing::error!("{:?}", err);
                                    Err(format!("{err}"))
                                }
                            }
                        }
                    }
                }
                AIQuery::DropStore {
                    store,
                    error_if_not_exists,
                } => {
                    let drop_store_params = db_params::DropStoreParams::builder()
                        .store(store.to_string())
                        .error_if_not_exists(error_if_not_exists)
                        .tracing_id(parent_id.clone())
                        .build();
                    match self.db_client.drop_store(drop_store_params).await {
                        Ok(_) => self
                            .store_handler
                            .drop_store(store, error_if_not_exists)
                            .map(AIServerResponse::Del)
                            .map_err(|e| e.to_string()),
                        Err(err) => {
                            tracing::error!("{:?}", err);
                            Err(format!("{err}"))
                        }
                    }
                }
                AIQuery::CreatePredIndex { store, predicates } => {
                    let create_pred_index_params = db_params::CreatePredIndexParams::builder()
                        .store(store.to_string())
                        .predicates(predicates)
                        .tracing_id(parent_id.clone())
                        .build();
                    match self
                        .db_client
                        .create_pred_index(create_pred_index_params)
                        .await
                    {
                        Ok(res) => {
                            if let ServerResponse::CreateIndex(num) = res {
                                Ok(AIServerResponse::CreateIndex(num))
                            } else {
                                tracing::error!("{:?}", res);
                                Err(AIProxyError::UnexpectedDBResponse(format!("{:?}", res))
                                    .to_string())
                            }
                        }
                        Err(err) => {
                            tracing::error!("{:?}", err);
                            Err(format!("{err}"))
                        }
                    }
                }
                AIQuery::CreateNonLinearAlgorithmIndex {
                    store,
                    non_linear_indices,
                } => {
                    let create_non_linear_algo_params =
                        db_params::CreateNonLinearAlgorithmIndexParams::builder()
                            .store(store.to_string())
                            .non_linear_indices(non_linear_indices)
                            .tracing_id(parent_id.clone())
                            .build();
                    match self
                        .db_client
                        .create_non_linear_algorithm_index(create_non_linear_algo_params)
                        .await
                    {
                        Ok(res) => {
                            if let ServerResponse::CreateIndex(num) = res {
                                Ok(AIServerResponse::CreateIndex(num))
                            } else {
                                tracing::error!("{:?}", res);
                                Err(AIProxyError::UnexpectedDBResponse(format!("{:?}", res))
                                    .to_string())
                            }
                        }
                        Err(err) => {
                            tracing::error!("{:?}", err);
                            Err(format!("{err}"))
                        }
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
                    {
                        let drop_pred_index_params = db_params::DropPredIndexParams::builder()
                            .store(store.to_string())
                            .predicates(predicates)
                            .error_if_not_exists(error_if_not_exists)
                            .tracing_id(parent_id.clone())
                            .build();
                        match self.db_client.drop_pred_index(drop_pred_index_params).await {
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
                }
                AIQuery::DropNonLinearAlgorithmIndex {
                    store,
                    non_linear_indices,
                    error_if_not_exists,
                } => {
                    let drop_non_linear_algorithm_index_params =
                        db_params::DropNonLinearAlgorithmIndexParams::builder()
                            .store(store.to_string())
                            .non_linear_indices(non_linear_indices)
                            .error_if_not_exists(error_if_not_exists)
                            .tracing_id(parent_id.clone())
                            .build();
                    match self
                        .db_client
                        .drop_non_linear_algorithm_index(drop_non_linear_algorithm_index_params)
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
                    let get_pred_params = db_params::GetPredParams::builder()
                        .store(store.to_string())
                        .condition(condition)
                        .tracing_id(parent_id.clone())
                        .build();
                    match self.db_client.get_pred(get_pred_params).await {
                        Ok(res) => {
                            if let ServerResponse::Get(response) = res {
                                // conversion to store input here
                                let output = self
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
                    preprocess_action,
                    execution_provider,
                } => {
                    let repr = self
                        .store_handler
                        .get_ndarray_repr_for_store(
                            &store,
                            search_input,
                            &self.model_manager,
                            preprocess_action,
                            execution_provider,
                        )
                        .await;
                    match repr {
                        Ok(store_key) => {
                            let get_sim_n_params = db_params::GetSimNParams::builder()
                                .store(store.to_string())
                                .search_input(store_key)
                                .closest_n(closest_n.into())
                                .algorithm(algorithm)
                                .condition(condition)
                                .tracing_id(parent_id.clone())
                                .build();
                            match self.db_client.get_sim_n(get_sim_n_params).await {
                                Ok(res) => {
                                    if let ServerResponse::GetSimN(response) = res {
                                        let (store_key_input, similarities): (Vec<_>, Vec<_>) =
                                            response
                                                .into_par_iter()
                                                .map(|(a, b, c)| ((a, b), c))
                                                .unzip();
                                        Ok(AIServerResponse::GetSimN(
                                            self.store_handler
                                                .store_key_val_to_store_input_val(store_key_input)
                                                .into_par_iter()
                                                .zip(similarities.into_par_iter())
                                                .map(|((a, b), c)| (a, b, c))
                                                .collect(),
                                        ))
                                    } else {
                                        Err(AIProxyError::UnexpectedDBResponse(format!(
                                            "{:?}",
                                            res
                                        ))
                                        .to_string())
                                    }
                                }
                                Err(err) => {
                                    tracing::error!("{:?}", err);
                                    Err(format!("{err}"))
                                }
                            }
                        }
                        Err(err) => {
                            tracing::error!("{:?}", err);
                            Err(AIProxyError::StandardError(err.to_string()).to_string())
                        }
                    }
                }
                AIQuery::PurgeStores => {
                    let destoryed = self.store_handler.purge_stores();
                    Ok(AIServerResponse::Del(destoryed))
                }
                AIQuery::ListClients => {
                    Ok(AIServerResponse::ClientList(self.client_handler.list()))
                }
                AIQuery::GetKey { store, keys } => {
                    let metadata_values: HashSet<MetadataValue> =
                        keys.into_par_iter().map(|value| value.into()).collect();
                    let get_key_condition = PredicateCondition::Value(Predicate::In {
                        key: AHNLICH_AI_RESERVED_META_KEY.clone(),
                        value: metadata_values,
                    });

                    let get_pred_params = db_params::GetPredParams::builder()
                        .store(store.to_string())
                        .condition(get_key_condition)
                        .tracing_id(parent_id.clone())
                        .build();

                    match self.db_client.get_pred(get_pred_params).await {
                        Ok(res) => {
                            if let ServerResponse::Get(response) = res {
                                // conversion to store input here
                                let output = self
                                    .store_handler
                                    .store_key_val_to_store_input_val(response);
                                Ok(AIServerResponse::Get(output))
                            } else {
                                tracing::error!("{:?}", res);
                                Err(AIProxyError::UnexpectedDBResponse(format!("{:?}", res))
                                    .to_string())
                            }
                        }
                        Err(err) => {
                            tracing::error!("{:?}", err);
                            Err(format!("{err}"))
                        }
                    }
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
