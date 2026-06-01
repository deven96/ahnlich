use crate::cli::ServerConfig;
use crate::engine::store::StoresSnapshot;
use crate::replication::DbStateMachine;
use crate::server::cluster::{ClusterRuntime, DbRaft};
use ahnlich_replication::admin::ClusterAdmin;
use ahnlich_replication::network::GrpcRaftService;
use ahnlich_replication::proto::cluster_admin::cluster_admin_service_server::ClusterAdminServiceServer;
use ahnlich_replication::proto::raft_internal::raft_internal_service_server::RaftInternalServiceServer;
use ahnlich_replication::storage::StateMachineStore;
use std::future::Future;
use std::io::Result as IoResult;
use std::sync::Arc;
use std::sync::Mutex;
use task_manager::{BlockingTask, Task, TaskManager, TaskState};
use tokio::time::{Duration, sleep};
use utils::connection_layer::trace_with_parent;
use utils::server::ListenerStreamOrAddress;
use utils::size_calculation::SizeCalculationHandler;

struct ClusterServiceTask {
    listener: ListenerStreamOrAddress,
    raft: Arc<DbRaft>,
}

impl std::fmt::Debug for ClusterServiceTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("ClusterServiceTask").finish_non_exhaustive()
    }
}

#[async_trait::async_trait]
impl BlockingTask for ClusterServiceTask {
    fn task_name(&self) -> String {
        "ahnlich-db-cluster".to_owned()
    }

    async fn run(
        mut self,
        shutdown_signal: std::pin::Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>,
    ) {
        let listener_stream = if let ListenerStreamOrAddress::ListenerStream(stream) = self.listener
        {
            stream
        } else {
            panic!("cluster listener must be of type listener stream");
        };

        self.listener = ListenerStreamOrAddress::Address(
            listener_stream
                .as_ref()
                .local_addr()
                .expect("Could not get cluster local address"),
        );

        let raft_service = RaftInternalServiceServer::new(GrpcRaftService::new(self.raft.clone()));
        let admin_service = ClusterAdminServiceServer::new(ClusterAdmin::new(self.raft.clone()));

        let _ = tonic::transport::Server::builder()
            .trace_fn(trace_with_parent)
            .add_service(raft_service)
            .add_service(admin_service)
            .serve_with_incoming_shutdown(listener_stream, shutdown_signal)
            .await;
    }
}

#[derive(Debug, Clone)]
struct ClusterSizeCalculationTask {
    state_machine: StateMachineStore<crate::replication::DbTypeConfig, DbStateMachine>,
    interval_ms: u64,
}

#[async_trait::async_trait]
impl Task for ClusterSizeCalculationTask {
    fn task_name(&self) -> String {
        "cluster_size_calculation".to_owned()
    }

    async fn run(&self) -> TaskState {
        sleep(Duration::from_millis(self.interval_ms)).await;
        if let Err(err) = self.state_machine.with_handler(|handler| {
            StoresSnapshot::new(handler.store_handler().get_stores()).recalculate_all_sizes();
        }) {
            log::error!("Failed to recalculate clustered store sizes: {err}");
            return TaskState::Break;
        }
        TaskState::Continue
    }
}

#[derive(Debug, Clone)]
struct ReplicationFailureWatcher {
    failure_state: Arc<ahnlich_replication::storage::ReplicationFailureState>,
    task_manager: Arc<TaskManager>,
}

#[async_trait::async_trait]
impl Task for ReplicationFailureWatcher {
    fn task_name(&self) -> String {
        "db_replication_failure_watcher".to_owned()
    }

    async fn run(&self) -> TaskState {
        sleep(Duration::from_millis(100)).await;
        if self.failure_state.failed() {
            log::error!(
                "Replication failure detected, shutting down DB server: {:?}",
                self.failure_state.reason()
            );
            self.task_manager.cancel_all();
            TaskState::Break
        } else {
            TaskState::Continue
        }
    }
}

struct SnapshotTriggerTask {
    raft: Arc<DbRaft>,
    interval_ms: u64,
    last_seen_applied: Mutex<Option<u64>>,
}

impl std::fmt::Debug for SnapshotTriggerTask {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SnapshotTriggerTask")
            .field("interval_ms", &self.interval_ms)
            .finish_non_exhaustive()
    }
}

impl SnapshotTriggerTask {
    fn new(raft: Arc<DbRaft>, interval_ms: u64) -> Self {
        Self {
            raft,
            interval_ms,
            last_seen_applied: Mutex::new(None),
        }
    }
}

#[async_trait::async_trait]
impl Task for SnapshotTriggerTask {
    fn task_name(&self) -> String {
        "db_snapshot_trigger".to_owned()
    }

    async fn run(&self) -> TaskState {
        sleep(Duration::from_millis(self.interval_ms)).await;

        let last_applied = self
            .raft
            .metrics()
            .borrow()
            .last_applied
            .map(|log_id| log_id.index);
        let Some(last_applied) = last_applied else {
            return TaskState::Continue;
        };

        let should_snapshot = {
            let mut guard = self
                .last_seen_applied
                .lock()
                .expect("snapshot trigger mutex poisoned");
            let should = guard.is_none_or(|seen| seen != last_applied);
            if should {
                *guard = Some(last_applied);
            }
            should
        };

        if should_snapshot && let Err(err) = self.raft.trigger().snapshot().await {
            log::warn!("Failed to trigger DB snapshot: {err}");
        }

        TaskState::Continue
    }
}

pub(crate) async fn spawn_cluster_tasks(
    config: &ServerConfig,
    task_manager: &Arc<TaskManager>,
    cluster: &ClusterRuntime,
) -> IoResult<()> {
    task_manager
        .spawn_task_loop(ClusterSizeCalculationTask {
            state_machine: cluster.state_machine.clone(),
            interval_ms: config.common.size_calculation_interval,
        })
        .await;
    task_manager
        .spawn_task_loop(ReplicationFailureWatcher {
            failure_state: cluster.failure_state.clone(),
            task_manager: task_manager.clone(),
        })
        .await;
    task_manager
        .spawn_task_loop(SnapshotTriggerTask::new(
            cluster.raft.clone(),
            config.cluster_snapshot_interval,
        ))
        .await;
    task_manager
        .spawn_blocking(ClusterServiceTask {
            listener: cluster.take_cluster_listener()?,
            raft: cluster.raft.clone(),
        })
        .await;

    Ok(())
}
