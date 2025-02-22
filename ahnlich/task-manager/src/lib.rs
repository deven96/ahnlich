use std::future::Future;

use log::info;
use tokio::{select, signal};
/// TaskManager is a struct to achieve multiple things
///
/// - Spawn multiple long running "tasks" that expect to be run in a loop
/// - Break the task loop when one of the following happens:
///     - SIGterm or SIGint is received
///     - External cancellation token is triggered
/// - Define necessary cleanup when a task is called upon for cancellation
///
/// TaskManager extends the tokio_utils::task::TaskTracker to ensure that every task loop is being tracked
/// and the tasks have the ability to perform any necessary cleanup before ending
///
/// Tasks must implement Task trait and be movable across threads i.e Sync + Send + 'static
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

#[derive(Debug, Clone)]
pub struct TaskManager {
    cancellation_token: CancellationToken,
    task_tracker: TaskTracker,
}

#[derive(Debug, Clone, Copy)]
pub enum TaskState {
    Continue,
    Break,
}

#[async_trait::async_trait]
pub trait Task {
    fn task_name(&self) -> String;
    async fn run(&self) -> TaskState;
    // optional cleanup upon task exit
    async fn cleanup(&self) {}
}

#[async_trait::async_trait]
pub trait BlockingTask {
    fn task_name(&self) -> String;
    async fn run(
        self,
        shutdown_signal: std::pin::Pin<Box<dyn Future<Output = ()> + Send + Sync + 'static>>,
    );
}

impl Default for TaskManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            cancellation_token: CancellationToken::new(),
            task_tracker: TaskTracker::new(),
        }
    }

    pub async fn spawn_blocking(&self, task: impl BlockingTask + Send + Sync + 'static) {
        let task_name = task.task_name();
        let cancellation_token = self.cancellation_token.clone();

        let run = async move {
            let cancel_fut = Box::pin(async move {
                select! {
                    biased;
                    _ = signal::ctrl_c() => {
                        info!("Received Ctrl-C signal, cancelling [{task_name}] task");
                    }
                    _ = cancellation_token.cancelled() => {
                        info!("Received Cancellation token signal, cancelling [{task_name}] task");
                    }
                }
            });
            task.run(cancel_fut).await
        };

        self.task_tracker.spawn(run);
    }

    pub async fn spawn_task_loop(&self, task: impl Task + Send + Sync + 'static) {
        let task_name = task.task_name();
        let task_name_copy = task_name.clone();
        let cancellation_token = self.cancellation_token.clone();
        self.task_tracker.spawn(async move {
            loop {
                select! {
                    // We use biased selection as it would order our futures according to physical
                    // arrangements below
                    // We want shutdown signals to always be checked for first, hence the arrangements
                    biased;

                    _ = signal::ctrl_c() => {
                        info!("Received Ctrl-C signal, cancelling [{task_name}] task");
                        task.cleanup().await;
                        break;
                    }
                    _ = cancellation_token.cancelled() => {
                        info!("Received Cancellation token signal, cancelling [{task_name}] task");
                        task.cleanup().await;
                        break;
                    }
                    state = task.run() => {
                        match state {
                            TaskState::Continue => continue,
                            TaskState::Break => break,
                        }
                    }
                }
            }
        });
        log::debug!("Spawned task {}", task_name_copy);
    }

    pub fn cancel_all(&self) {
        self.cancellation_token.cancel()
    }

    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancellation_token.clone()
    }

    pub async fn wait(&self) {
        self.task_tracker.close();
        self.task_tracker.wait().await
    }

    pub fn task_count(&self) -> usize {
        self.task_tracker.len()
    }
}
