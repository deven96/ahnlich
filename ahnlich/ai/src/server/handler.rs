use crate::cli::AIProxyConfig;
use crate::engine::store::AIStoreHandler;
use crate::server::task::AIProxyTask;
use ahnlich_types::db::ConnectedClient;
use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use std::{io::Result as IoResult, sync::Arc};
use tokio::io::BufReader;
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio_graceful::Shutdown;
use tracing::Instrument;
use utils::{client::ClientHandler, protocol::AhnlichProtocol};

use ahnlich_client_rs::db::{DbClient, DbConnManager};
use deadpool::managed::Pool;

pub struct AIProxyServer<'a> {
    listener: TcpListener,
    config: &'a AIProxyConfig,
    client_handler: Arc<ClientHandler>,
    store_handler: Arc<AIStoreHandler>,
    shutdown_token: Shutdown,
    db_client: Arc<DbClient>,
}

impl<'a> AIProxyServer<'a> {
    pub async fn new(config: &'a AIProxyConfig) -> IoResult<Self> {
        let shutdown_token = Shutdown::default();
        Self::build(config, shutdown_token).await
    }

    pub async fn build(config: &'a AIProxyConfig, shutdown_token: Shutdown) -> IoResult<Self> {
        let listener =
            tokio::net::TcpListener::bind(format!("{}:{}", &config.host, &config.port)).await?;
        // Enable tracing
        if config.enable_tracing {
            let otel_url = config
                .otel_endpoint
                .to_owned()
                .unwrap_or("http://127.0.0.1:4317".to_string());
            tracer::init_tracing("ahnlich-db", Some(&config.log_level), &otel_url)
        }
        let write_flag = Arc::new(AtomicBool::new(false));
        let db_client = Self::build_db_client(config).await;
        let store_handler = AIStoreHandler::new(write_flag.clone());
        let client_handler = Arc::new(ClientHandler::new(config.maximum_clients));
        Ok(Self {
            listener,
            shutdown_token,
            client_handler,
            store_handler: Arc::new(store_handler),
            config,
            db_client: Arc::new(db_client),
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
                    //  - Convert the incoming bincode in a chunked manner to a Vec<AIQuery>
                    //  - Use store_handler to process the queries
                    //  - Block new incoming connections on shutdown by no longer accepting and then
                    //  cancelling existing ServerTask or forcing them to run to completion

                    if let Some(connected_client) = self.client_handler.connect(stream.peer_addr()?) {
                        let mut task = self.create_task(stream, server_addr, connected_client)?;
                        shutdown_guard.spawn_task_fn(|guard| async move {
                            if let Err(e) = task.process(guard).instrument(tracing::info_span!("server_task").or_current()).await {
                                tracing::error!("Error handling connection: {}", e)
                            };
                        });
                    }
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

    fn create_task(
        &self,
        stream: TcpStream,
        server_addr: SocketAddr,
        connected_client: ConnectedClient,
    ) -> IoResult<AIProxyTask> {
        let reader = BufReader::new(stream);
        // add client to client_handler
        Ok(AIProxyTask {
            reader,
            server_addr,
            connected_client,
            maximum_message_size: self.config.message_size as u64,
            // "inexpensive" to clone handlers they can be passed around in an Arc
            client_handler: self.client_handler.clone(),
            store_handler: self.store_handler.clone(),
            db_client: self.db_client.clone(),
        })
    }

    async fn build_db_client(config: &'a AIProxyConfig) -> DbClient {
        let manager = DbConnManager::new(config.db_host.clone(), config.db_port);
        let pool = Pool::builder(manager)
            .max_size(config.db_client_pool_size)
            .build()
            .expect("Cannot establish connection to the Database");

        DbClient::new_with_pool(pool)
    }
}
