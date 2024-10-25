use crate::cli::server::ModelConfig;
use crate::cli::AIProxyConfig;
use crate::engine::ai::models::Model;
use crate::engine::store::AIStoreHandler;
use crate::manager::ModelManager;
use crate::server::task::AIProxyTask;
use ahnlich_types::client::ConnectedClient;
use std::error::Error;
use std::io::Result as IoResult;
use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use task_manager::Task;
use task_manager::TaskManager;
use task_manager::TaskState;
use tokio::io::BufReader;
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use utils::client::ClientHandler;
use utils::persistence::Persistence;
use utils::server::AhnlichServerUtils;
use utils::server::ServerUtilsConfig;

use ahnlich_client_rs::db::{DbClient, DbConnManager};
use deadpool::managed::Pool;

const SERVICE_NAME: &str = "ahnlich-ai";

#[derive(Debug, Clone)]
pub struct AIProxyServer {
    listener: Arc<TcpListener>,
    config: AIProxyConfig,
    client_handler: Arc<ClientHandler>,
    store_handler: Arc<AIStoreHandler>,
    task_manager: Arc<TaskManager>,
    db_client: Arc<DbClient>,
    model_manager: Arc<ModelManager>,
}

#[async_trait::async_trait]
impl Task for AIProxyServer {
    fn task_name(&self) -> String {
        "ai-listener".to_string()
    }

    async fn run(&self) -> TaskState {
        if let Ok((stream, connect_addr)) = self.listener.accept().await {
            if let Some(connected_client) = self
                .client_handler
                .connect(stream.peer_addr().expect("Could not get peer addr"))
            {
                log::info!("Connecting to {}", connect_addr);
                let task = self.create_task(
                    stream,
                    self.local_addr().expect("Could not get server addr"),
                    connected_client,
                );
                self.task_manager.spawn_task_loop(task).await;
            }
        }
        TaskState::Continue
    }
}

impl AhnlichServerUtils for AIProxyServer {
    type PersistenceTask = AIStoreHandler;

    fn config(&self) -> ServerUtilsConfig {
        ServerUtilsConfig {
            service_name: SERVICE_NAME,
            persist_location: &self.config.common.persist_location,
            persistence_interval: self.config.common.persistence_interval,
            allocator_size: self.config.common.allocator_size,
            threadpool_size: self.config.common.threadpool_size,
        }
    }

    fn cancellation_token(&self) -> CancellationToken {
        self.task_manager.cancellation_token()
    }

    fn store_handler(&self) -> &Arc<Self::PersistenceTask> {
        &self.store_handler
    }

    fn task_manager(&self) -> Arc<TaskManager> {
        self.task_manager.clone()
    }
}

impl AIProxyServer {
    pub async fn new(config: AIProxyConfig) -> Result<Self, Box<dyn Error>> {
        Self::build(config).await
    }

    pub async fn build(config: AIProxyConfig) -> Result<Self, Box<dyn Error>> {
        // Enable log and tracing
        tracer::init_log_or_trace(
            config.common.enable_tracing,
            SERVICE_NAME,
            &config.common.otel_endpoint,
            &config.common.log_level,
        );
        let listener =
            tokio::net::TcpListener::bind(format!("{}:{}", &config.common.host, &config.port))
                .await?;
        let write_flag = Arc::new(AtomicBool::new(false));
        let db_client = Self::build_db_client(&config).await;
        let mut store_handler =
            AIStoreHandler::new(write_flag.clone(), config.supported_models.clone());
        if let Some(ref persist_location) = config.common.persist_location {
            match Persistence::load_snapshot(persist_location) {
                Err(e) => {
                    log::error!("Failed to load snapshot from persist location {e}");
                    if config.common.fail_on_startup_if_persist_load_fails {
                        return Err(Box::new(e));
                    }
                }
                Ok(snapshot) => {
                    store_handler.use_snapshot(snapshot);
                }
            }
        };
        let client_handler = Arc::new(ClientHandler::new(config.common.maximum_clients));
        let task_manager = Arc::new(TaskManager::new());
        let mut models: Vec<Model> = Vec::with_capacity(config.supported_models.len());
        for supported_model in &config.supported_models {
            let mut model: Model = supported_model.into();
            model.setup_provider(&config.model_cache_location);
            // TODO (HAKSOAT): Handle if download fails or verification check fails
            model.get()?;
            models.push(model);
        }

        let model_config = ModelConfig::from(&config);
        let model_manager = ModelManager::new(model_config, task_manager.clone()).await?;

        Ok(Self {
            listener: Arc::new(listener),
            client_handler,
            store_handler: Arc::new(store_handler),
            config,
            db_client: Arc::new(db_client),
            task_manager,
            model_manager: Arc::new(model_manager),
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

    fn create_task(
        &self,
        stream: TcpStream,
        server_addr: SocketAddr,
        connected_client: ConnectedClient,
    ) -> AIProxyTask {
        let reader = BufReader::new(stream);
        // add client to client_handler
        AIProxyTask {
            reader: Arc::new(Mutex::new(reader)),
            server_addr,
            connected_client,
            maximum_message_size: self.config.common.message_size as u64,
            // "inexpensive" to clone handlers they can be passed around in an Arc
            client_handler: self.client_handler.clone(),
            store_handler: self.store_handler.clone(),
            db_client: self.db_client.clone(),
            model_manager: self.model_manager.clone(),
        }
    }

    pub fn local_addr(&self) -> IoResult<SocketAddr> {
        self.listener.local_addr()
    }
}
