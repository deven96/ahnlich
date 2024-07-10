use ahnlich_client_rs::db::DbClient;
use ahnlich_types::ai::{AIQuery, AIServerQuery, AIServerResponse, AIServerResult};
use ahnlich_types::bincode::BinCodeSerAndDeserResponse;
use ahnlich_types::db::{ConnectedClient, ServerInfo, ServerResponse};
use ahnlich_types::keyval::{StoreInput, StoreValue};
use ahnlich_types::metadata::{MetadataKey, MetadataValue};
use ahnlich_types::predicate::{Predicate, PredicateCondition};
use ahnlich_types::version::VERSION;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use std::vec;
use tokio::io::BufReader;
use tokio::net::TcpStream;
use utils::client::ClientHandler;
use utils::protocol::AhnlichProtocol;

use crate::engine::store::AIStoreHandler;
use crate::error::AIProxyError;
use crate::AHNLICH_RESERVED_AI_META;

#[derive(Debug)]
pub(super) struct AIProxyTask {
    pub(super) server_addr: SocketAddr,
    pub(super) reader: BufReader<TcpStream>,
    pub(super) client_handler: Arc<ClientHandler>,
    pub(super) store_handler: Arc<AIStoreHandler>,
    pub(super) connected_client: ConnectedClient,
    pub(super) maximum_message_size: u64,
    pub(super) db_client: Arc<DbClient>,
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
    fn reader(&mut self) -> &mut BufReader<TcpStream> {
        &mut self.reader
    }

    async fn handle(&self, queries: Vec<AIQuery>) -> AIServerResult {
        let mut result = AIServerResult::with_capacity(queries.len());
        for query in queries {
            result.push(match query {
                AIQuery::Ping => Ok(AIServerResponse::Pong),
                AIQuery::ListStores => Ok(AIServerResponse::StoreList(
                    self.store_handler.list_stores(),
                )),
                AIQuery::InfoServer => Ok(AIServerResponse::InfoServer(self.server_info())),
                AIQuery::CreateStore {
                    r#type,
                    store,
                    model,
                    mut predicates,
                    non_linear_indices,
                } => {
                    // TODO: Make sure you edit predicates to include _ahnlich_hash_id which is the
                    // key mapping to the original input in the store, also validate predicates not
                    // to have the _ahnlich_hash_id reserve key
                    if predicates.contains(&MetadataKey::new(AHNLICH_RESERVED_AI_META.to_string()))
                    {
                        return AIServerResult::from_error(format!(
                            "Cannot use {} keyword",
                            AHNLICH_RESERVED_AI_META
                        ));
                    }
                    predicates.insert(MetadataKey::new(AHNLICH_RESERVED_AI_META.to_string()));
                    match self
                        .db_client
                        .create_store(
                            store.clone(),
                            model.embedding_size(),
                            predicates,
                            non_linear_indices,
                            false,
                        )
                        .await
                    {
                        Err(err) => Err(err.to_string()),
                        Ok(_) => self
                            .store_handler
                            .create_store(store, r#type, model)
                            .map(|_| AIServerResponse::Unit)
                            .map_err(|e| e.to_string()),
                    }
                }

                AIQuery::Set { store, inputs } => {
                    let mut db_inputs = Vec::new();
                    let mut delete_hashset = HashSet::new();
                    let default_key = MetadataKey::new(AHNLICH_RESERVED_AI_META.to_string());
                    for (store_input, store_value) in inputs.into_iter() {
                        let store_value = self.store_handler.store_input_to_store_key_val(
                            &store,
                            store_input,
                            &store_value,
                        );
                        match store_value {
                            Ok(val) => {
                                delete_hashset.insert(val.1[&default_key].clone());
                                db_inputs.push(val)
                            }
                            Err(err) => return AIServerResult::from_error(err.to_string()),
                        }
                    }

                    let delete_condition = PredicateCondition::Value(Predicate::In {
                        key: MetadataKey::new(AHNLICH_RESERVED_AI_META.to_string()),
                        value: delete_hashset,
                    });
                    if let Err(err) = self
                        .db_client
                        .del_pred(store.clone(), delete_condition)
                        .await
                    {
                        return AIServerResult::from_error(err.to_string());
                    };

                    match self.db_client.set(store, db_inputs).await {
                        Ok(res) => {
                            if let ServerResponse::Set(upsert) = res {
                                Ok(AIServerResponse::Set(upsert))
                            } else {
                                Err(AIProxyError::UnexpectedDBResponse(format!("{:?}", res))
                                    .to_string())
                            }
                        }
                        Err(err) => Err(format!("{err}")),
                    }
                }

                AIQuery::DelKey { store, key } => {
                    let metadata_value: MetadataValue = key.into();
                    let delete_condition = PredicateCondition::Value(Predicate::In {
                        key: MetadataKey::new(AHNLICH_RESERVED_AI_META.to_string()),
                        value: HashSet::from_iter([metadata_value]),
                    });

                    match self.db_client.del_pred(store, delete_condition).await {
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
                    .drop_store(store.clone(), error_if_not_exists)
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
                    if predicates.contains(&MetadataKey::new(AHNLICH_RESERVED_AI_META.to_string()))
                    {
                        return AIServerResult::from_error(format!(
                            "Cannot use {} keyword",
                            AHNLICH_RESERVED_AI_META
                        ));
                    }

                    match self.db_client.create_pred_index(store, predicates).await {
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
                    let default_metadata = MetadataKey::new(AHNLICH_RESERVED_AI_META.to_string());
                    if predicates.contains(&default_metadata) {
                        let _ = predicates.remove(&default_metadata);
                    }
                    match self
                        .db_client
                        .drop_pred_index(store, predicates, error_if_not_exists)
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
                    match self.db_client.get_pred(store, condition).await {
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
                    if let Ok(store_key) = self
                        .store_handler
                        .get_ndarray_repr_for_store(&store, &search_input)
                    {
                        match self
                            .db_client
                            .get_sim_n(store, store_key, closest_n, algorithm, condition)
                            .await
                        {
                            Ok(res) => {
                                if let ServerResponse::GetSimN(response) = res {
                                    // conversion to store input here
                                    let mut output = Vec::new();

                                    for (store_key, store_value, sim) in response.into_iter() {
                                        let temp =
                                            self.store_handler.store_key_val_to_store_input_val(
                                                vec![(store_key, store_value)],
                                            );

                                        if let Some(valid_result) = temp.first().take() {
                                            let valid_result = valid_result.to_owned();
                                            output.push((valid_result.0, valid_result.1, sim))
                                        }
                                    }

                                    Ok(AIServerResponse::GetSimN(output))
                                } else {
                                    Err(AIProxyError::UnexpectedDBResponse(format!("{:?}", res))
                                        .to_string())
                                }
                            }
                            Err(err) => Err(format!("{err}")),
                        }
                    } else {
                        return AIServerResult::from_error(
                            AIProxyError::StandardError("Failed to get store".to_string())
                                .to_string(),
                        );
                    }
                }
            })
        }
        result
    }
}

impl AIProxyTask {
    fn server_info(&self) -> ServerInfo {
        ServerInfo {
            address: format!("{}", self.server_addr),
            version: *VERSION,
            r#type: ahnlich_types::ServerType::AI,
            limit: 20,
            remaining: 29,
        }
    }
}

impl Drop for AIProxyTask {
    fn drop(&mut self) {
        self.client_handler.disconnect(&self.connected_client);
    }
}
