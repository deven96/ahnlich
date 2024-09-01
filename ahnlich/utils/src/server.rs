use crate::allocator::GLOBAL_ALLOCATOR;
use crate::client::ClientHandler;
use crate::persistence::AhnlichPersistenceUtils;
use crate::persistence::Persistence;
use crate::protocol::AhnlichProtocol;
use ahnlich_types::client::ConnectedClient;
use async_trait::async_trait;
use std::net::SocketAddr;
use std::sync::atomic::AtomicBool;
use std::{io::Result as IoResult, sync::Arc};
use task_manager::TaskManager;
use tokio::net::{TcpListener, TcpStream};
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
pub trait AhnlichServerUtils: Sized + Send + Sync + 'static {
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
    ) -> Self::Task;

    fn local_addr(&self) -> IoResult<SocketAddr> {
        self.listener().local_addr()
    }

    fn cancellation_token(&self) -> CancellationToken;

    fn task_manager(&self) -> Arc<TaskManager>;

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
        let task_manager = self.task_manager();
        let inner_task_manager = self.task_manager();
        let waiting_task_manager = self.task_manager();

        if let Some(persist_location) = self.config().persist_location {
            let persistence_task = Persistence::task(
                self.write_flag(),
                self.config().persistence_interval,
                persist_location,
                self.store_handler().get_snapshot(),
            );
            task_manager
                .spawn_task_loop(
                    |shutdown_guard| async move { persistence_task.monitor(shutdown_guard).await },
                    "persistence".to_string(),
                )
                .await;
        };
        task_manager
            .spawn_task_loop(move |shutdown_guard| async move {
                loop {
                    tokio::select! {
                        biased;

                        _ = shutdown_guard.is_cancelled() => {
                            break;
                        }
                        Ok((stream, connect_addr)) = self.listener().accept() => {
                            if let Some(connected_client) = self
                                .client_handler()
                                    .connect(stream.peer_addr().expect("Could not get peer addr"))
                            {
                                log::info!("Connecting to {}", connect_addr);
                                let task = self.create_task(stream, server_addr, connected_client);
                                inner_task_manager.spawn_task_loop(|shutdown_guard| async move {

                                    task.process(shutdown_guard)
                                        .instrument(
                                            tracing::info_span!("server_task").or_current(),
                                        )
                                        .await
                                },
                                format!("{}-listener", connect_addr),
                                ).await;
                            }
                        }
                    };
                }
            },
        "listener".to_string()
            )
        .await;
        waiting_task_manager.wait().await;
        tracer::shutdown_tracing();
        log::info!("Shutdown complete");
        Ok(())
    }
}
