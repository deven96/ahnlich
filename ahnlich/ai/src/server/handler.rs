use crate::cli::AIProxyConfig;
use crate::engine::store::AIStoreHandler;
use crate::server::task::AIProxyTask;
use ahnlich_types::client::ConnectedClient;
use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;
use std::{io::Result as IoResult, sync::Arc};
use tokio::io::BufReader;
use tokio::net::{TcpListener, TcpStream};
use tokio_graceful::Shutdown;
use tokio_util::sync::CancellationToken;
use utils::client::ClientHandler;
use utils::persistence::Persistence;
use utils::server::AhnlichServerUtils;
use utils::server::ServerUtilsConfig;

use ahnlich_client_rs::db::{DbClient, DbConnManager};
use deadpool::managed::Pool;

const SERVICE_NAME: &'static str = "ahnlich-ai";

pub struct AIProxyServer {
    listener: TcpListener,
    config: AIProxyConfig,
    client_handler: Arc<ClientHandler>,
    store_handler: Arc<AIStoreHandler>,
    shutdown_token: Shutdown,
    cancellation_token: CancellationToken,
    db_client: Arc<DbClient>,
}

impl AhnlichServerUtils for AIProxyServer {
    type Task = AIProxyTask;
    type PersistenceTask = AIStoreHandler;

    fn config(&self) -> ServerUtilsConfig {
        ServerUtilsConfig {
            service_name: SERVICE_NAME,
            persist_location: &self.config.persist_location,
            persistence_interval: self.config.persistence_interval,
            allocator_size: self.config.allocator_size,
        }
    }

    fn cancellation_token(&self) -> &CancellationToken {
        &self.cancellation_token
    }

    fn listener(&self) -> &TcpListener {
        &self.listener
    }

    fn client_handler(&self) -> &Arc<ClientHandler> {
        &self.client_handler
    }

    fn store_handler(&self) -> &Arc<Self::PersistenceTask> {
        &self.store_handler
    }

    fn shutdown_token(&self) -> &Shutdown {
        &self.shutdown_token
    }

    fn shutdown_token_owned(self) -> Shutdown {
        self.shutdown_token
    }

    fn create_task(
        &self,
        stream: TcpStream,
        server_addr: SocketAddr,
        connected_client: ConnectedClient,
    ) -> IoResult<Self::Task> {
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
}

impl AIProxyServer {
    pub async fn new(config: AIProxyConfig) -> IoResult<Self> {
        let shutdown_token = Shutdown::default();
        Self::build(config, shutdown_token).await
    }

    pub async fn build(config: AIProxyConfig, shutdown_token: Shutdown) -> IoResult<Self> {
        // Enable log and tracing
        tracer::init_log_or_trace(
            config.enable_tracing,
            SERVICE_NAME,
            &config.otel_endpoint,
            &config.log_level,
        );
        let listener =
            tokio::net::TcpListener::bind(format!("{}:{}", &config.host, &config.port)).await?;
        let write_flag = Arc::new(AtomicBool::new(false));
        let db_client = Self::build_db_client(&config).await;
        let mut store_handler =
            AIStoreHandler::new(write_flag.clone(), config.supported_models.clone());
        if let Some(ref persist_location) = config.persist_location {
            match Persistence::load_snapshot(persist_location) {
                Err(e) => {
                    log::error!("Failed to load snapshot from persist location {e}");
                    if config.fail_on_startup_if_persist_load_fails {
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::Other,
                            e.to_string(),
                        ));
                    }
                }
                Ok(snapshot) => {
                    store_handler.use_snapshot(snapshot);
                }
            }
        };
        let client_handler = Arc::new(ClientHandler::new(config.maximum_clients));

        Ok(Self {
            listener,
            shutdown_token,
            client_handler,
            store_handler: Arc::new(store_handler),
            cancellation_token: CancellationToken::new(),
            config,
            db_client: Arc::new(db_client),
        })
    }

    async fn build_db_client(config: &AIProxyConfig) -> DbClient {
        let manager = DbConnManager::new(config.db_host.clone(), config.db_port);
        let pool = Pool::builder(manager)
            .max_size(config.db_client_pool_size)
            .build()
            .expect("Cannot establish connection to the Database");

        DbClient::new_with_pool(pool)
    }
}
