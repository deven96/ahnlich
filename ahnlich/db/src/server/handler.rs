use super::task::ServerTask;
use crate::cli::ServerConfig;
use crate::engine::store::StoreHandler;
use ahnlich_types::client::ConnectedClient;
use std::io::Result as IoResult;
use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use task_manager::TaskManager;
use tokio::io::BufReader;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use utils::server::AhnlichServerUtils;
use utils::server::ServerUtilsConfig;
use utils::{client::ClientHandler, persistence::Persistence};

const SERVICE_NAME: &str = "ahnlich-db";

#[derive(Debug, Clone)]
pub struct Server {
    listener: Arc<TcpListener>,
    store_handler: Arc<StoreHandler>,
    client_handler: Arc<ClientHandler>,
    task_manager: Arc<TaskManager>,
    config: ServerConfig,
}

impl AhnlichServerUtils for Server {
    type Task = ServerTask;
    type PersistenceTask = StoreHandler;

    fn config(&self) -> ServerUtilsConfig {
        ServerUtilsConfig {
            service_name: SERVICE_NAME,
            persist_location: &self.config.persist_location,
            persistence_interval: self.config.persistence_interval,
            allocator_size: self.config.allocator_size,
        }
    }

    fn cancellation_token(&self) -> CancellationToken {
        self.task_manager.cancellation_token()
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

    fn task_manager(&self) -> Arc<TaskManager> {
        self.task_manager.clone()
    }

    fn create_task(
        &self,
        stream: TcpStream,
        server_addr: SocketAddr,
        connected_client: ConnectedClient,
    ) -> Self::Task {
        let reader = BufReader::new(stream);
        // add client to client_handler
        ServerTask {
            reader: Arc::new(Mutex::new(reader)),
            server_addr,
            connected_client,
            maximum_message_size: self.config.message_size as u64,
            // "inexpensive" to clone handlers they can be passed around in an Arc
            client_handler: self.client_handler.clone(),
            store_handler: self.store_handler.clone(),
        }
    }
}

impl Server {
    /// creates a server while injecting a shutdown_token
    pub async fn new_with_config(config: &ServerConfig) -> IoResult<Self> {
        let listener =
            tokio::net::TcpListener::bind(format!("{}:{}", &config.host, &config.port)).await?;
        let write_flag = Arc::new(AtomicBool::new(false));
        let client_handler = Arc::new(ClientHandler::new(config.maximum_clients));
        let mut store_handler = StoreHandler::new(write_flag.clone());
        if let Some(persist_location) = &config.persist_location {
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
        Ok(Self {
            listener: Arc::new(listener),
            store_handler: Arc::new(store_handler),
            client_handler,
            task_manager: Arc::new(TaskManager::new()),
            config: config.clone(),
        })
    }

    /// initializes a server using server configuration
    pub async fn new(config: &ServerConfig) -> IoResult<Self> {
        // Enable log and tracing
        tracer::init_log_or_trace(
            config.enable_tracing,
            SERVICE_NAME,
            &config.otel_endpoint,
            &config.log_level,
        );
        Self::new_with_config(config).await
    }
}
