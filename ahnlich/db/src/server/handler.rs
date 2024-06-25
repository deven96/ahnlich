use super::task::ServerTask;
use crate::cli::ServerConfig;
use crate::client::ClientHandler;
use crate::engine::store::StoreHandler;
use crate::persistence::PersistenceTask;
use cap::Cap;
use std::alloc;
use std::io::Result as IoResult;
use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::BufReader;
use tokio::net::TcpListener;
use tokio::net::TcpStream;
use tokio::select;
use tokio_graceful::Shutdown;
use tracing::Instrument;

#[global_allocator]
pub(super) static ALLOCATOR: Cap<alloc::System> = Cap::new(alloc::System, usize::max_value());

pub(crate) struct Server<'a> {
    listener: TcpListener,
    shutdown_token: Shutdown,
    store_handler: Arc<StoreHandler>,
    client_handler: Arc<ClientHandler>,
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
            tracer::init_tracing("ahnlich-db", Some(&config.log_level), &otel_url)
        }
        let shutdown_token = Shutdown::default();
        let write_flag = Arc::new(AtomicBool::new(false));
        let client_handler = Arc::new(ClientHandler::new());
        let mut store_handler = StoreHandler::new(write_flag.clone());
        if let Some(persist_location) = &config.persist_location {
            if let Err(e) = store_handler.load_snapshot(persist_location) {
                tracing::error!("Failed to load snapshot from persist location {e}");
            }
            // spawn the persistence task
            let mut persistence_task = PersistenceTask::new(
                write_flag,
                config.persistence_interval,
                persist_location,
                store_handler.get_stores(),
            );
            shutdown_token
                .spawn_task_fn(|guard| async move { persistence_task.monitor(guard).await });
        };
        Ok(Self {
            listener,
            shutdown_token,
            store_handler: Arc::new(store_handler),
            client_handler,
            config,
        })
    }

    #[cfg(test)]
    pub fn write_flag(&self) -> Arc<AtomicBool> {
        self.store_handler.write_flag()
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
                        if let Err(e) = task.process(guard).instrument(tracing::info_span!("server_task").or_current()).await {
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
