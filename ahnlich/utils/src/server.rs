use crate::allocator::GLOBAL_ALLOCATOR;
use crate::client::ClientHandler;
use crate::persistence::AhnlichPersistenceUtils;
use crate::persistence::Persistence;
use crate::protocol::AhnlichProtocol;
use ahnlich_types::client::ConnectedClient;
use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;
use std::time::Duration;
use std::{io::Result as IoResult, sync::Arc};
use tokio::net::{TcpListener, TcpStream};
use tokio::select;
use tokio_graceful::Shutdown;
use tokio_util::sync::CancellationToken;
use tracing::Instrument;

#[derive(Debug)]
pub struct ServerUtilsConfig<'a> {
    pub service_name: &'static str,
    // persistence stuff
    pub persistence_interval: u64,
    pub persist_location: &'a Option<std::path::PathBuf>,
    // global allocator
    pub allocator_size: usize,
}

#[async_trait]
pub trait AhnlichServerUtils: Sized {
    type Task: AhnlichProtocol + Send + Sync + 'static;
    type PersistenceTask: AhnlichPersistenceUtils;

    fn config(&self) -> ServerUtilsConfig;

    fn listener(&self) -> &TcpListener;
    fn client_handler(&self) -> &Arc<ClientHandler>;
    fn store_handler(&self) -> &Arc<Self::PersistenceTask>;
    fn write_flag(&self) -> Arc<AtomicBool> {
        self.store_handler().write_flag()
    }

    fn create_task(
        &self,
        stream: TcpStream,
        server_addr: SocketAddr,
        connected_client: ConnectedClient,
    ) -> IoResult<Self::Task>;

    fn local_addr(&self) -> IoResult<SocketAddr> {
        self.listener().local_addr()
    }

    fn shutdown_token(&self) -> &Shutdown;
    fn shutdown_token_owned(self) -> Shutdown;

    fn cancellation_token(&self) -> &CancellationToken;

    /// stops all tasks and performs cleanup
    async fn shutdown(self) {
        let service_name = self.config().service_name;
        // TODO: Add cleanup for instance persistence
        // just cancelling the token alone does not give enough time for each task to shutdown,
        // there must be other measures in place to ensure proper cleanup
        if self
            .shutdown_token_owned()
            .shutdown_with_limit(Duration::from_secs(10))
            .await
            .is_err()
        {
            log::error!("{}: shutdown took longer than timeout", service_name);
        };
        tracer::shutdown_tracing();
    }

    /// Runs through several processes to start up the server
    /// - Sets global allocator cap
    /// - Spawns Persistence listeneer thread
    /// - Accepts incoming connections to the listener and processes streams
    /// - Listens for ctrl_c signal to trigger spawned tasks cancellation
    /// - Cancellation triggers clean up of loggers and tracers
    async fn start(self) -> IoResult<()> {
        let service_name = self.config().service_name;
        let global_allocator_cap = self.config().allocator_size;
        GLOBAL_ALLOCATOR
            .set_limit(global_allocator_cap)
            .unwrap_or_else(|_| panic!("Could not set up {service_name} with allocator_size"));

        log::debug!("Set max size for global allocator to: {global_allocator_cap}");
        let server_addr = self.local_addr()?;
        if let Some(persist_location) = self.config().persist_location {
            let persistence_task = Persistence::task(
                self.write_flag(),
                self.config().persistence_interval,
                persist_location,
                self.store_handler().get_snapshot(),
            );
            self.shutdown_token()
                .spawn_task_fn(|guard| async move { persistence_task.monitor(guard).await });
        };
        loop {
            let shutdown_guard = self.shutdown_token().guard();
            select! {
                // We use biased selection as it would order our futures according to physical
                // arrangements below
                // We want shutdown signals to always be checked for first, hence the arrangements
                biased;

                // shutdown handler from Ctrl+C signals
                _ = shutdown_guard.cancelled() => {
                    // need to drop the shutdown guard used here else self.shutdown times out
                    log::info!("Kill signal received for shutdown");
                    drop(shutdown_guard);
                    self.shutdown().await;
                    break Ok(());
                }
                // shutdown handler from external cancellation tokens
                _ = self.cancellation_token().cancelled() => {
                    log::info!("External thread triggered cancellation token");
                    drop(shutdown_guard);
                    self.shutdown().await;
                    break Ok(());
                }
                Ok((stream, connect_addr)) = self.listener().accept() => {
                    //  - Spawn a tokio task to handle the command while holding on to a reference to self
                    //  - Convert the incoming bincode in a chunked manner to a Vec<AIQuery>
                    //  - Use store_handler to process the queries
                    //  - Block new incoming connections on shutdown by no longer accepting and then
                    //  cancelling existing ServerTask or forcing them to run to completion

                    if let Some(connected_client) = self.client_handler().connect(stream.peer_addr()?) {
                        log::info!("Connecting to {}", connect_addr);
                        let mut task = self.create_task(stream, server_addr, connected_client)?;
                        shutdown_guard.spawn_task_fn(|guard| async move {
                            if let Err(e) = task.process(guard).instrument(tracing::info_span!("server_task").or_current()).await {
                                log::error!("Error handling connection: {}", e)
                            };
                        });
                    }
                }
            }
        }
    }
}
