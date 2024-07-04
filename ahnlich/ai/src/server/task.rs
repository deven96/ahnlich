use ahnlich_client_rs::db::DbClient;
use ahnlich_types::ai::{AIQuery, AIServerQuery, AIServerResponse, AIServerResult};
use ahnlich_types::db::{ConnectedClient, ServerInfo};
use ahnlich_types::version::VERSION;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::BufReader;
use tokio::net::TcpStream;
use utils::client::ClientHandler;
use utils::protocol::AhnlichProtocol;

use crate::engine::store::AIStoreHandler;

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
                    match self
                        .db_client
                        .create_store(
                            store.clone(),
                            model.embedding_size().clone(),
                            predicates,
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
