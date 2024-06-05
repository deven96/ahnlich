#![allow(dead_code)]
#![allow(clippy::size_of_ref)]
mod algorithm;
pub mod cli;
mod client;
mod engine;
mod errors;
mod network;
mod storage;
use crate::cli::ServerConfig;
use crate::client::ClientHandler;
use crate::engine::store::StoreHandler;
use cap::Cap;
use std::alloc;
use std::io::Result as IoResult;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::select;
use tokio_graceful::Shutdown;
use tokio_graceful::ShutdownGuard;
use types::bincode::BinCodeSerAndDeser;
use types::bincode::LENGTH_HEADER_SIZE;
use types::query::Query;
use types::query::ServerQuery;
use types::server::ConnectedClient;
use types::server::ServerInfo;
use types::server::ServerResponse;
use types::server::ServerResult;

#[global_allocator]
static ALLOCATOR: Cap<alloc::System> = Cap::new(alloc::System, usize::max_value());

pub struct Server<'a> {
    listener: TcpListener,
    store_handler: Arc<StoreHandler>,
    client_handler: Arc<ClientHandler>,
    shutdown_token: Shutdown,
    config: &'a ServerConfig,
}

impl<'a> Server<'a> {
    /// initializes a server using server configuration
    pub async fn new(config: &'a ServerConfig) -> IoResult<Self> {
        ALLOCATOR
            .set_limit(config.allocator_size)
            .expect("Could not set up server with allocator_size");
        let listener =
            tokio::net::TcpListener::bind(format!("{}:{}", &config.host, &config.port)).await?;
        // Enable tracing
        if config.enable_tracing {
            let otel_url = config
                .otel_endpoint
                .to_owned()
                .unwrap_or("http://127.0.0.1:4317".to_string());
            tracer::init_tracing("Ahnlich-db", Some(&config.log_level), &otel_url)
        }
        // TODO: replace with rules to retrieve store handler from persistence if persistence exist
        let store_handler = Arc::new(StoreHandler::new());
        let client_handler = Arc::new(ClientHandler::new());
        let shutdown_token = Shutdown::default();
        Ok(Self {
            listener,
            store_handler,
            client_handler,
            shutdown_token,
            config,
        })
    }

    /// starts accepting connections using the listener and processing the incoming streams
    ///
    /// listens for a ctrl_c signals to cancel spawned tasks
    pub async fn start(self) -> IoResult<()> {
        let server_addr = self.local_addr()?;
        loop {
            let shutdown_guard = self.shutdown_token.guard();
            select! {
                _ = shutdown_guard.cancelled() => {
                    // need to drop the shutdown guard used here else self.shutdown times out
                    drop(shutdown_guard);
                    self.shutdown().await;
                    break Ok(());
                }
                Ok((stream, connect_addr)) = self.listener.accept() => {
                    tracing::info!("Connecting to {}", connect_addr);
                    //  - Spawn a tokio task to handle the command while holding on to a reference to self
                    //  - Convert the incoming bincode in a chunked manner to a Vec<Query>
                    //  - Use store_handler to process the queries
                    //  - Block new incoming connections on shutdown by no longer accepting and then
                    //  cancelling existing ServerTask or forcing them to run to completion

                    let mut task = self.create_task(stream, server_addr)?;
                    shutdown_guard.spawn_task_fn(|guard| async move {
                        if let Err(e) = task.process(guard).await {
                            tracing::error!("Error handling connection: {}", e)
                        };
                    });
                }
            }
        }
    }

    /// stops all tasks and performs cleanup
    pub async fn shutdown(self) {
        // TODO: Add cleanup for instance persistence
        // just cancelling the token alone does not give enough time for each task to shutdown,
        // there must be other measures in place to ensure proper cleanup
        if self
            .shutdown_token
            .shutdown_with_limit(Duration::from_secs(10))
            .await
            .is_err()
        {
            tracing::error!("SERVER: shutdown took longer than timeout");
        }
    }

    pub fn local_addr(&self) -> IoResult<SocketAddr> {
        self.listener.local_addr()
    }

    fn create_task(&self, stream: TcpStream, server_addr: SocketAddr) -> IoResult<ServerTask> {
        let connected_client = self.client_handler.connect(stream.peer_addr()?);
        let reader = BufReader::new(stream);
        // add client to client_handler
        Ok(ServerTask {
            reader,
            server_addr,
            connected_client,
            maximum_message_size: self.config.message_size as u64,
            // "inexpensive" to clone handlers they can be passed around in an Arc
            client_handler: self.client_handler.clone(),
            store_handler: self.store_handler.clone(),
        })
    }
}

#[derive(Debug)]
struct ServerTask {
    server_addr: SocketAddr,
    reader: BufReader<TcpStream>,
    store_handler: Arc<StoreHandler>,
    client_handler: Arc<ClientHandler>,
    connected_client: ConnectedClient,
    maximum_message_size: u64,
}

impl ServerTask {
    fn prefix_log(&self, message: impl std::fmt::Display) -> String {
        format!("ClIENT [{}]: {}", self.connected_client.address, message)
    }

    #[tracing::instrument]
    /// processes messages from a stream
    async fn process(&mut self, shutdown_token: ShutdownGuard) -> IoResult<()> {
        let mut length_buf = [0u8; LENGTH_HEADER_SIZE];

        loop {
            select! {
                _ = shutdown_token.cancelled() => {
                    tracing::debug!("{}", self.prefix_log("Cancelling stream as server is shutting down"));
                    break;
                }
                res = self.reader.read_exact(&mut length_buf) => {
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
                            // cap the message size to be of length 1MiB
                            let data_length = u64::from_be_bytes(length_buf);
                            if data_length > self.maximum_message_size {
                                tracing::error!("{}", self.prefix_log(format!("Message cannot exceed {} bytes, configure `message_size` for higher", self.maximum_message_size)));
                                tracing::error_span!("Messsage exceeds max size");
                                continue
                            }
                            let mut data = Vec::new();
                            if data.try_reserve(data_length as usize).is_err() {
                                tracing::error!("{}", self.prefix_log(format!("Failed to reserve buffer of length {data_length}")));
                                continue
                            };
                            data.resize(data_length as usize, 0u8);
                            self.reader.read_exact(&mut data).await?;
                            // TODO: Add trace here to catch whenever queries could not be deserialized at all
                            if let Ok(queries) = ServerQuery::deserialize(false, &data) {
                                // TODO: Pass in store_handler and use to respond to queries
                                let results = self.handle(queries.into_inner());
                                if let Ok(binary_results) = results.serialize() {
                                    self.reader.get_mut().write_all(&binary_results).await?;
                                }
                            } else {

                                tracing::error_span!("Could not deserialize client message as server query");
                                tracing::error!("{}", self.prefix_log("Could not deserialize client message as server query"));
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

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
            version: types::VERSION.to_string(),
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

#[cfg(test)]
mod fixtures;
