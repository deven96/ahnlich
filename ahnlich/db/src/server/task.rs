use crate::engine::store::StoreHandler;
use crate::errors::ServerError;
use crate::server::handler::ALLOCATOR;
use std::io::Error;
use std::io::Result as IoResult;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::select;
use tokio_graceful::ShutdownGuard;
use types::bincode::BinCodeSerAndDeser;
use types::bincode::LENGTH_HEADER_SIZE;
use types::bincode::MAGIC_BYTES;
use types::bincode::VERSION_LENGTH;
use types::query::Query;
use types::query::ServerQuery;
use types::server::ConnectedClient;
use types::server::ServerInfo;
use types::server::ServerResponse;
use types::server::ServerResult;
use types::version::Version;
use types::version::VERSION;
use utils::client::ClientHandler;

#[derive(Debug)]
pub(super) struct ServerTask {
    pub(super) server_addr: SocketAddr,
    pub(super) reader: BufReader<TcpStream>,
    pub(super) store_handler: Arc<StoreHandler>,
    pub(super) client_handler: Arc<ClientHandler>,
    pub(super) connected_client: ConnectedClient,
    pub(super) maximum_message_size: u64,
}

impl ServerTask {
    fn prefix_log(&self, message: impl std::fmt::Display) -> String {
        format!("ClIENT [{}]: {}", self.connected_client.address, message)
    }

    /// processes messages from a stream
    pub(super) async fn process(&mut self, shutdown_guard: ShutdownGuard) -> IoResult<()> {
        let mut magic_bytes_buf = [0u8; MAGIC_BYTES.len()];
        let mut version_buf = [0u8; VERSION_LENGTH];
        let mut length_buf = [0u8; LENGTH_HEADER_SIZE];

        loop {
            select! {
                _ = shutdown_guard.cancelled() => {
                    tracing::debug!("{}", self.prefix_log("Cancelling stream as server is shutting down"));
                    break;
                }
                res = self.reader.read_exact(&mut magic_bytes_buf) => {
                    match res {
                        // reader was closed
                        Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                            tracing::debug!("{}", self.prefix_log("Hung up on buffered stream"));
                            break;
                        }
                        Err(e) => {
                            tracing::error!("{}", self.prefix_log(format!("Error reading from task buffered stream {e}")));
                        }
                        Ok(_) => {
                            if magic_bytes_buf != MAGIC_BYTES {
                                return Err(Error::other("Invalid request stream"));
                            }
                            self.reader.read_exact(&mut version_buf).await?;
                            let version = Version::deserialize_magic_bytes(&version_buf).map_err(|a| Error::other(format!("Unable to parse version chunk {a}")))?;
                            if !VERSION.is_compatible(&version) {
                                return Err(Error::other(format!("Incompatible versions, Server: {:?}, Client {version:?}", *VERSION)));
                            }
                            // cap the message size to be of length 1MiB
                            self.reader.read_exact(&mut length_buf).await?;
                            let data_length = u64::from_le_bytes(length_buf);
                            if data_length > self.maximum_message_size {
                                tracing::error!("{}", self.prefix_log(format!("Message cannot exceed {} bytes, configure `message_size` for higher", self.maximum_message_size)));
                                break
                            }
                            let mut data = Vec::new();
                            if data.try_reserve(data_length as usize).is_err() {
                                tracing::error!("{}", self.prefix_log(format!("Failed to reserve buffer of length {data_length}")));
                                break
                            };
                            data.resize(data_length as usize, 0u8);
                            self.reader.read_exact(&mut data).await?;
                            // TODO: Add trace here to catch whenever queries could not be deserialized at all
                            match ServerQuery::deserialize(&data) {
                                Ok(queries) => {
                                tracing::debug!("Got Queries {:?}", queries);
                                // TODO: Pass in store_handler and use to respond to queries
                                let results = self.handle(queries.into_inner());
                                if let Ok(binary_results) = results.serialize() {
                                    self.reader.get_mut().write_all(&binary_results).await?;
                                    tracing::debug!("Sent Response of length {}, {:?}", binary_results.len(), binary_results);
                                }
                            },
                            Err(e) =>{
                                tracing::error!("{} {e}", self.prefix_log("Could not deserialize client message as server query"));
                                let deserialize_error = ServerResult::from_error(format!("{}", ServerError::QueryDeserializeError("".into()))).serialize().expect("Could not serialize deserialize error");
                                self.reader.get_mut().write_all(&deserialize_error).await?;
                            }
                        }
                    }
                    }
                }
            }
        }
        Ok(())
    }

    #[tracing::instrument]
    fn handle(&self, queries: Vec<Query>) -> ServerResult {
        let mut result = ServerResult::with_capacity(queries.len());
        for query in queries {
            result.push(match query {
                Query::Ping => Ok(ServerResponse::Pong),
                Query::InfoServer => Ok(ServerResponse::InfoServer(self.server_info())),
                Query::ListClients => Ok(ServerResponse::ClientList(self.client_handler.list())),
                Query::ListStores => {
                    Ok(ServerResponse::StoreList(self.store_handler.list_stores()))
                }
                Query::CreateStore {
                    store,
                    dimension,
                    create_predicates,
                    error_if_exists,
                } => self
                    .store_handler
                    .create_store(
                        store,
                        dimension,
                        create_predicates.into_iter().collect(),
                        error_if_exists,
                    )
                    .map(|_| ServerResponse::Unit)
                    .map_err(|e| format!("{e}")),
                Query::CreateIndex { store, predicates } => self
                    .store_handler
                    .create_index(&store, predicates.into_iter().collect())
                    .map(ServerResponse::CreateIndex)
                    .map_err(|e| format!("{e}")),
                Query::DropStore {
                    store,
                    error_if_not_exists,
                } => self
                    .store_handler
                    .drop_store(store, error_if_not_exists)
                    .map(ServerResponse::Del)
                    .map_err(|e| format!("{e}")),
                Query::DropIndex {
                    store,
                    error_if_not_exists,
                    predicates,
                } => self
                    .store_handler
                    .drop_index_in_store(
                        &store,
                        predicates.into_iter().collect(),
                        error_if_not_exists,
                    )
                    .map(ServerResponse::Del)
                    .map_err(|e| format!("{e}")),
                Query::Set { store, inputs } => self
                    .store_handler
                    .set_in_store(&store, inputs)
                    .map(ServerResponse::Set)
                    .map_err(|e| format!("{e}")),
                Query::GetKey { store, keys } => self
                    .store_handler
                    .get_key_in_store(&store, keys)
                    .map(ServerResponse::Get)
                    .map_err(|e| format!("{e}")),
                Query::GetPred { store, condition } => self
                    .store_handler
                    .get_pred_in_store(&store, &condition)
                    .map(ServerResponse::Get)
                    .map_err(|e| format!("{e}")),
                Query::GetSimN {
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
                Query::DelKey { store, keys } => self
                    .store_handler
                    .del_key_in_store(&store, keys)
                    .map(ServerResponse::Del)
                    .map_err(|e| format!("{e}")),
                Query::DelPred { store, condition } => self
                    .store_handler
                    .del_pred_in_store(&store, &condition)
                    .map(ServerResponse::Del)
                    .map_err(|e| format!("{e}")),
            })
        }
        result
    }

    fn server_info(&self) -> ServerInfo {
        ServerInfo {
            address: format!("{}", self.server_addr),
            version: *VERSION,
            r#type: types::server::ServerType::Database,
            limit: ALLOCATOR.limit(),
            remaining: ALLOCATOR.remaining(),
        }
    }
}

impl Drop for ServerTask {
    fn drop(&mut self) {
        self.client_handler.disconnect(&self.connected_client);
    }
}
