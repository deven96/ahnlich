use ahnlich_client_rs::db::DbClient;
use ahnlich_types::ai::{AIQuery, AIServerQuery, AIServerResponse, AIServerResult};
use ahnlich_types::bincode::BinCodeSerAndDeserResponse;
use ahnlich_types::db::{ConnectedClient, ServerInfo, ServerResponse};
use ahnlich_types::metadata::MetadataKey;
use ahnlich_types::predicate::{Predicate, PredicateCondition};
use ahnlich_types::version::VERSION;
use std::collections::HashSet;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::BufReader;
use tokio::net::TcpStream;
use utils::client::ClientHandler;
use utils::protocol::AhnlichProtocol;

use crate::engine::store::AIStoreHandler;
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
                    predicates,
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
                    let mut predicates = HashSet::from_iter(predicates.into_iter());
                    predicates.insert(MetadataKey::new(AHNLICH_RESERVED_AI_META.to_string()));
                    match self
                        .db_client
                        .create_store(store.clone(), model.embedding_size(), predicates, false)
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
                    let default_key = MetadataKey::new(AHNLICH_RESERVED_AI_META.to_string());
                    for (store_input, store_value) in inputs.into_iter() {
                        let store_value = self.store_handler.store_input_to_store_key_val(
                            &store,
                            store_input,
                            &store_value,
                        );
                        match store_value {
                            Err(err) => return AIServerResult::from_error(err.to_string()),
                            Ok(val) => db_inputs.push(val),
                        }
                    }
                    let store_values: Vec<_> = db_inputs
                        .clone()
                        .into_iter()
                        .map(|(_, metavalue)| metavalue)
                        .filter(|x| (x.contains_key(&default_key)))
                        .map(|x| {
                            x.get(&default_key)
                                .expect("Cannot get metadata value")
                                .to_owned()
                        })
                        .collect();

                    let delete_condition = PredicateCondition::Value(Predicate::In {
                        key: MetadataKey::new(AHNLICH_RESERVED_AI_META.to_string()),
                        value: HashSet::from_iter(store_values),
                    });
                    if let Err(err) = self
                        .db_client
                        .del_pred(store.clone(), delete_condition)
                        .await
                    {
                        return AIServerResult::from_error(err.to_string());
                    };

                    self.db_client
                        .set(store, db_inputs)
                        .await
                        .map(|res| {
                            if let ServerResponse::Set(upsert) = res {
                                AIServerResponse::Set(upsert)
                            } else {
                                AIServerResponse::Pong
                            }
                        })
                        .map_err(|e| format!("{e}"))
                }

                _ => Ok(AIServerResponse::Pong),
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
