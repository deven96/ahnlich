use crate::allocator::GLOBAL_ALLOCATOR;
use crate::parallel;
use crate::persistence::AhnlichPersistenceUtils;
use crate::persistence::Persistence;
use async_trait::async_trait;
use std::sync::atomic::AtomicBool;
use std::{io::Result as IoResult, sync::Arc};
use task_manager::Task;
use task_manager::TaskManager;
use tokio_util::sync::CancellationToken;

#[derive(Debug)]
pub struct ServerUtilsConfig<'a> {
    pub service_name: &'static str,
    // persistence stuff
    pub persistence_interval: u64,
    pub persist_location: &'a Option<std::path::PathBuf>,
    // global allocator
    pub allocator_size: usize,
    pub threadpool_size: usize,
}

#[async_trait]
pub trait AhnlichServerUtils: Task + Sized + Send + Sync + 'static {
    type PersistenceTask: AhnlichPersistenceUtils;

    fn config(&self) -> ServerUtilsConfig;

    fn store_handler(&self) -> &Arc<Self::PersistenceTask>;
    fn write_flag(&self) -> Arc<AtomicBool> {
        self.store_handler().write_flag()
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
        parallel::init_threadpool(self.config().threadpool_size);
        let task_manager = self.task_manager();

        if let Some(persist_location) = self.config().persist_location {
            let persistence_task = Persistence::task(
                self.write_flag(),
                self.config().persistence_interval,
                persist_location,
                self.store_handler().get_snapshot(),
            );
            task_manager.spawn_task_loop(persistence_task).await;
        };
        task_manager.spawn_task_loop(self).await;
        task_manager.wait().await;
        tracer::shutdown_tracing();
        log::info!("Shutdown complete");
        Ok(())
    }
}
