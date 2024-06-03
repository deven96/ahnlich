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
use std::io::Result as IoResult;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::select;
use tokio::signal;
use tokio_util::sync::CancellationToken;
use types::bincode::BinCodeSerAndDeser;
use types::bincode::LENGTH_HEADER_SIZE;
use types::query::Query;
use types::query::ServerQuery;
use types::server::ConnectedClient;
use types::server::ServerInfo;
use types::server::ServerResponse;
use types::server::ServerResult;

#[derive(Debug)]
pub struct Server {
    listener: TcpListener,
    store_handler: Arc<StoreHandler>,
    client_handler: Arc<ClientHandler>,
    shutdown_token: CancellationToken,
}

impl Server {
    /// initializes a server using server configuration
    pub async fn new(config: &ServerConfig) -> IoResult<Self> {
        let listener =
            tokio::net::TcpListener::bind(format!("{}:{}", &config.host, &config.port)).await?;
        // TODO: replace with rules to retrieve store handler from persistence if persistence exist
        let store_handler = Arc::new(StoreHandler::new());
        let client_handler = Arc::new(ClientHandler::new());
        let shutdown_token = CancellationToken::new();
        Ok(Self {
            listener,
            store_handler,
            client_handler,
            shutdown_token,
        })
    }

    /// starts accepting connections using the listener and processing the incoming streams
    ///
    /// listens for a ctrl_c signals to cancel spawned tasks
    pub async fn start(&self) -> IoResult<()> {
        let server_addr = self.local_addr()?;
        loop {
            select! {
                _ = signal::ctrl_c() => {
                    self.shutdown();
                    break Ok(());
                }
                Ok((stream, connect_addr)) = self.listener.accept() => {
                    log::info!("Connecting to {}", connect_addr);
                    //  - Spawn a tokio task to handle the command while holding on to a reference to self
                    //  - Convert the incoming bincode in a chunked manner to a Vec<Query>
                    //  - Use store_handler to process the queries
                    //  - Block new incoming connections on shutdown by no longer accepting and then
                    //  cancelling existing ServerTask or forcing them to run to completion

                    let mut task = self.create_task(stream, server_addr)?;
                    tokio::spawn(async move {
                        if let Err(e) = task.process().await {
                            log::error!("Error handling connection: {}", e)
                        };
                    });
                }
            }
        }
    }

    /// stops all tasks and performs cleanup
    pub fn shutdown(&self) {
        // TODO: Add cleanup for instance persistence
        self.shutdown_token.cancel()
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
            shutdown_token: self.shutdown_token.clone(),
            // "inexpensive" to clone handlers they can be passed around in an Arc
            client_handler: self.client_handler.clone(),
            store_handler: self.store_handler.clone(),
        })
    }
}

#[derive(Debug)]
struct ServerTask {
    shutdown_token: CancellationToken,
    server_addr: SocketAddr,
    reader: BufReader<TcpStream>,
    store_handler: Arc<StoreHandler>,
    client_handler: Arc<ClientHandler>,
    connected_client: ConnectedClient,
}

impl ServerTask {
    /// processes messages from a stream
    async fn process(&mut self) -> IoResult<()> {
        let mut length_buf = [0u8; LENGTH_HEADER_SIZE];

        loop {
            select! {
                _ = self.shutdown_token.cancelled() => {
                    break;
                }
                res = self.reader.read_exact(&mut length_buf) => {
                    match res {
                        // reader was closed
                        Err(ref e) if e.kind() == std::io::ErrorKind::UnexpectedEof => {
                            log::debug!("Client {} hung up buffered stream", self.connected_client.address);
                            break;
                        }
                        Err(e) => {
                            log::error!("Error reading from task buffered stream: {}", e);
                        }
                        Ok(_) => {
                            let data_length = u64::from_be_bytes(length_buf);
                            let mut data = vec![0u8; data_length as usize];
                            self.reader.read_exact(&mut data).await?;
                            // TODO: Add trace here to catch whenever queries could not be deserialized at all
                            if let Ok(queries) = ServerQuery::deserialize(false, &data) {
                                // TODO: Pass in store_handler and use to respond to queries
                                let results = self.handle(queries.into_inner());
                                if let Ok(binary_results) = results.serialize() {
                                    self.reader.get_mut().write_all(&binary_results).await?;
                                }
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
                _ => Err("Response not implemented".to_string()),
            })
        }
        result
    }

    fn server_info(&self) -> ServerInfo {
        ServerInfo {
            address: format!("{}", self.server_addr),
            version: types::VERSION.to_string(),
            r#type: types::server::ServerType::Database,
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
