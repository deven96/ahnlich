#![allow(dead_code)]
#![allow(clippy::size_of_ref)]
mod algorithm;
pub mod cli;
mod engine;
mod errors;
mod network;
mod storage;
use crate::cli::ServerConfig;
use crate::engine::store::StoreHandler;
use std::io::Result as IoResult;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use types::bincode::BinCodeSerAndDeser;
use types::bincode::LENGTH_HEADER_SIZE;
use types::query::Query;
use types::query::ServerQuery;
use types::server::ServerResponse;
use types::server::ServerResult;

#[derive(Debug)]
pub struct Server {
    listener: TcpListener,
    store_handler: Arc<StoreHandler>,
}

impl Server {
    /// initializes a server using server configuration
    pub async fn new(config: &ServerConfig) -> IoResult<Self> {
        let listener =
            tokio::net::TcpListener::bind(format!("{}:{}", &config.host, &config.port)).await?;
        // TODO: replace with rules to retrieve store handler from persistence if persistence exist
        let store_handler = Arc::new(StoreHandler::new());
        Ok(Self {
            listener,
            store_handler,
        })
    }

    /// starts accepting connections using the listener and processing the incoming streams
    pub async fn start(&self) -> IoResult<()> {
        loop {
            let (stream, connect_addr) = self.listener.accept().await?;
            log::info!("Connecting to {}", connect_addr);
            // TODO
            //  - Spawn a tokio task to handle the command while holding on to a reference to self
            //  - Convert the incoming bincode in a chunked manner to a Vec<Query>
            //  - Use store_handler to process the queries
            //  - Block new incoming connections on shutdown by no longer accepting and then
            //  cancelling existing ServerTask or forcing them to run to completion

            // "inexpensive" to clone store handler as it is an Arc
            let task = ServerTask::new(stream, self.store_handler.clone());
            tokio::spawn(async move {
                if let Err(e) = task.process().await {
                    log::error!("Error handling connection: {}", e)
                };
            });
        }
    }

    pub fn local_addr(&self) -> IoResult<SocketAddr> {
        self.listener.local_addr()
    }
}

#[derive(Debug)]
struct ServerTask {
    stream: TcpStream,
    store_handler: Arc<StoreHandler>,
}

impl ServerTask {
    fn new(stream: TcpStream, store_handler: Arc<StoreHandler>) -> Self {
        Self {
            stream,
            store_handler,
        }
    }

    /// processes messages from a stream
    async fn process(self) -> IoResult<()> {
        self.stream.readable().await?;
        let mut reader = BufReader::new(self.stream);
        let mut length_buf = [0u8; LENGTH_HEADER_SIZE];
        loop {
            reader.read_exact(&mut length_buf).await?;
            let data_length = u64::from_be_bytes(length_buf);
            let mut data = vec![0u8; data_length as usize];
            reader.read_exact(&mut data).await?;
            // TODO: Add trace here to catch whenever queries could not be deserialized at all
            if let Ok(queries) = ServerQuery::deserialize(false, &data) {
                // TODO: Pass in store_handler and use to respond to queries
                let results = Self::handle(queries.into_inner());
                if let Ok(binary_results) = results.serialize() {
                    reader.get_mut().write_all(&binary_results).await?;
                }
            }
        }
    }

    fn handle(queries: Vec<Query>) -> ServerResult {
        let mut result = ServerResult::with_capacity(queries.len());
        for query in queries {
            result.push(match query {
                Query::Ping => Ok(ServerResponse::Pong),
                Query::InfoServer => Ok(ServerResponse::Unit),
                _ => Err("Response not implemented".to_string()),
            })
        }
        result
    }
}

#[cfg(test)]
mod fixtures;
