use super::task::ServerTask;
use crate::cli::ServerConfig;
use crate::engine::store::StoreHandler;
use ahnlich_types::client::ConnectedClient;
use std::io::Result as IoResult;
use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use tokio::io::BufReader;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio_graceful::Shutdown;
use tokio_util::sync::CancellationToken;
use utils::server::AhnlichServerUtils;
use utils::server::ServerUtilsConfig;
use utils::{client::ClientHandler, persistence::Persistence};

const SERVICE_NAME: &str = "ahnlich-db";

pub struct Server<'a> {
    listener: TcpListener,
    shutdown_token: Shutdown,
    store_handler: Arc<StoreHandler>,
    cancellation_token: CancellationToken,
    client_handler: Arc<ClientHandler>,
    config: &'a ServerConfig,
}

impl<'a> AhnlichServerUtils for Server<'a> {
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

impl<'a> Server<'a> {
    /// creates a server while injecting a shutdown_token
    pub async fn new_with_shutdown(
        config: &'a ServerConfig,
        shutdown_token: Shutdown,
    ) -> IoResult<Self> {
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
            listener,
            shutdown_token,
            store_handler: Arc::new(store_handler),
            client_handler,
            cancellation_token: CancellationToken::new(),
            config,
        })
    }
    /// initializes a server using server configuration
    pub async fn new(config: &'a ServerConfig) -> IoResult<Self> {
        let shutdown_token = Shutdown::default();
        // Enable log and tracing
        tracer::init_log_or_trace(
            config.enable_tracing,
            SERVICE_NAME,
            &config.otel_endpoint,
            &config.log_level,
        );
        Self::new_with_shutdown(config, shutdown_token).await
    }
}
