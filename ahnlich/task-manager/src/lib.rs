use log::info;
use std::future::Future;
use tokio::{select, signal};
/// TaskManager is a struct to achieve multiple things
///
/// - Spawn multiple long running "tasks" that expect to be run in a loop
/// - Break the task loop when one of the following happens:
///     - SIGterm or SIGint is received
///     - External cancellation token is triggered
///
/// TaskManager extends the tokio_utils::task::TaskTracker to ensure that every task loop is being tracked
/// and the tasks have the ability to perform any necessary cleanup before ending
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

#[derive(Debug, Clone)]
pub struct TaskManager {
    cancellation_token: CancellationToken,
    task_tracker: TaskTracker,
}

#[derive(Debug, Clone)]
pub struct TaskManagerGuard {
    cancellation_token: CancellationToken,
    task_name: String,
}

impl TaskManagerGuard {
    pub async fn is_cancelled(&self) {
        let task_name = self.task_name.clone();
        select! {
            // We use biased selection as it would order our futures according to physical
            // arrangements below
            // We want shutdown signals to always be checked for first, hence the arrangements
            biased;

            _ = signal::ctrl_c() => {
                info!("Received Ctrl-C signal, cancelling [{task_name}] task");
            }
            _ = self.cancellation_token.cancelled() => {
                info!("Received Cancellation token signal, cancelling [{task_name}] task");
            }
        }
    }
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

    fn guard(&self, task_name: String) -> TaskManagerGuard {
        TaskManagerGuard {
            task_name,
            cancellation_token: self.cancellation_token.clone(),
        }
    }

    pub async fn spawn_task_loop<T, F>(&self, task: F, task_name: String)
    where
        T: Future<Output = ()> + Send + 'static,
        F: FnOnce(TaskManagerGuard) -> T + Send + 'static,
    {
        let task_name_copy = task_name.clone();
        let guard = self.guard(task_name);

        self.task_tracker.spawn(task(guard));
        log::debug!("Spawned task {task_name_copy}",);
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
