use crate::engine::store::StoreHandler;
use ahnlich_types::client::ConnectedClient;
use ahnlich_types::db::{DBQuery, ServerDBQuery, ServerInfo, ServerResponse, ServerResult};
use ahnlich_types::version::VERSION;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::BufReader;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use utils::allocator::GLOBAL_ALLOCATOR;
use utils::client::ClientHandler;
use utils::protocol::AhnlichProtocol;

#[derive(Debug)]
pub struct ServerTask {
    pub(super) server_addr: SocketAddr,
    pub(super) reader: Arc<Mutex<BufReader<TcpStream>>>,
    pub(super) store_handler: Arc<StoreHandler>,
    pub(super) client_handler: Arc<ClientHandler>,
    pub(super) connected_client: ConnectedClient,
    pub(super) maximum_message_size: u64,
}

#[async_trait::async_trait]
impl AhnlichProtocol for ServerTask {
    type ServerQuery = ServerDBQuery;
    type ServerResponse = ServerResult;

    fn connected_client(&self) -> &ConnectedClient {
        &self.connected_client
    }
    fn maximum_message_size(&self) -> u64 {
        self.maximum_message_size
    }
    fn reader(&self) -> Arc<Mutex<BufReader<TcpStream>>> {
        self.reader.clone()
    }

    async fn handle(&self, queries: Vec<DBQuery>) -> ServerResult {
        let mut result = ServerResult::with_capacity(queries.len());
        for query in queries {
            result.push(match query {
                DBQuery::Ping => Ok(ServerResponse::Pong),
                DBQuery::InfoServer => Ok(ServerResponse::InfoServer(self.server_info())),
                DBQuery::ListClients => Ok(ServerResponse::ClientList(self.client_handler.list())),
                DBQuery::ListStores => {
                    Ok(ServerResponse::StoreList(self.store_handler.list_stores()))
                }
                DBQuery::CreateStore {
                    store,
                    dimension,
                    create_predicates,
                    non_linear_indices,
                    error_if_exists,
                } => self
                    .store_handler
                    .create_store(
                        store,
                        dimension,
                        create_predicates.into_iter().collect(),
                        non_linear_indices,
                        error_if_exists,
                    )
                    .map(|_| ServerResponse::Unit)
                    .map_err(|e| format!("{e}")),
                DBQuery::CreatePredIndex { store, predicates } => self
                    .store_handler
                    .create_pred_index(&store, predicates.into_iter().collect())
                    .map(ServerResponse::CreateIndex)
                    .map_err(|e| format!("{e}")),
                DBQuery::CreateNonLinearAlgorithmIndex {
                    store,
                    non_linear_indices,
                } => self
                    .store_handler
                    .create_non_linear_algorithm_index(&store, non_linear_indices)
                    .map(ServerResponse::CreateIndex)
                    .map_err(|e| format!("{e}")),
                DBQuery::DropStore {
                    store,
                    error_if_not_exists,
                } => self
                    .store_handler
                    .drop_store(store, error_if_not_exists)
                    .map(ServerResponse::Del)
                    .map_err(|e| format!("{e}")),
                DBQuery::DropPredIndex {
                    store,
                    error_if_not_exists,
                    predicates,
                } => self
                    .store_handler
                    .drop_pred_index_in_store(
                        &store,
                        predicates.into_iter().collect(),
                        error_if_not_exists,
                    )
                    .map(ServerResponse::Del)
                    .map_err(|e| format!("{e}")),
                DBQuery::DropNonLinearAlgorithmIndex {
                    store,
                    error_if_not_exists,
                    non_linear_indices,
                } => self
                    .store_handler
                    .drop_non_linear_algorithm_index(
                        &store,
                        non_linear_indices,
                        error_if_not_exists,
                    )
                    .map(ServerResponse::Del)
                    .map_err(|e| format!("{e}")),
                DBQuery::Set { store, inputs } => self
                    .store_handler
                    .set_in_store(&store, inputs)
                    .map(ServerResponse::Set)
                    .map_err(|e| format!("{e}")),
                DBQuery::GetKey { store, keys } => self
                    .store_handler
                    .get_key_in_store(&store, keys)
                    .map(ServerResponse::Get)
                    .map_err(|e| format!("{e}")),
                DBQuery::GetPred { store, condition } => self
                    .store_handler
                    .get_pred_in_store(&store, &condition)
                    .map(ServerResponse::Get)
                    .map_err(|e| format!("{e}")),
                DBQuery::GetSimN {
                    store,
                    search_input,
                    closest_n,
                    algorithm,
                    condition,
                } => self
                    .store_handler
                    .get_sim_in_store(&store, search_input, closest_n, algorithm, condition)
                    .map(ServerResponse::GetSimN)
                    .map_err(|e| format!("{e}")),
                DBQuery::DelKey { store, keys } => self
                    .store_handler
                    .del_key_in_store(&store, keys)
                    .map(ServerResponse::Del)
                    .map_err(|e| format!("{e}")),
                DBQuery::DelPred { store, condition } => self
                    .store_handler
                    .del_pred_in_store(&store, &condition)
                    .map(ServerResponse::Del)
                    .map_err(|e| format!("{e}")),
            })
        }
        result
    }
}

impl ServerTask {
    #[tracing::instrument(skip(self))]
    fn server_info(&self) -> ServerInfo {
        ServerInfo {
            address: format!("{}", self.server_addr),
            version: *VERSION,
            r#type: ahnlich_types::ServerType::Database,
            limit: GLOBAL_ALLOCATOR.limit(),
            remaining: GLOBAL_ALLOCATOR.remaining(),
        }
    }
}

impl Drop for ServerTask {
    fn drop(&mut self) {
        self.client_handler.disconnect(&self.connected_client);
    }
}
