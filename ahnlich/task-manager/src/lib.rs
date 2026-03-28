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

#[cfg(unix)]
async fn wait_for_shutdown_signal(task_name: String, cancellation_token: CancellationToken) {
    match signal::unix::signal(signal::unix::SignalKind::terminate()) {
        Ok(mut sigterm) => {
            select! {
                biased;
                _ = sigterm.recv() => {
                    info!("Received SIGTERM signal, cancelling [{task_name}] task");
                }
                _ = signal::ctrl_c() => {
                    info!("Received Ctrl-C signal, cancelling [{task_name}] task");
                }
                _ = cancellation_token.cancelled() => {
                    info!("Received Cancellation token signal, cancelling [{task_name}] task");
                }
            }
        }
        Err(err) => {
            log::warn!("Failed to register SIGTERM handler for [{task_name}] task: {err}");
            select! {
                biased;
                _ = signal::ctrl_c() => {
                    info!("Received Ctrl-C signal, cancelling [{task_name}] task");
                }
                _ = cancellation_token.cancelled() => {
                    info!("Received Cancellation token signal, cancelling [{task_name}] task");
                }
            }
        }
    }
}

#[cfg(not(unix))]
async fn wait_for_shutdown_signal(task_name: String, cancellation_token: CancellationToken) {
    select! {
        biased;
        _ = signal::ctrl_c() => {
            info!("Received Ctrl-C signal, cancelling [{task_name}] task");
        }
        _ = cancellation_token.cancelled() => {
            info!("Received Cancellation token signal, cancelling [{task_name}] task");
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

    pub async fn spawn_blocking(&self, task: impl BlockingTask + Send + Sync + 'static) {
        let task_name = task.task_name();
        let cancellation_token = self.cancellation_token.clone();

        let run = async move {
            let cancel_fut = Box::pin(wait_for_shutdown_signal(
                task_name.clone(),
                cancellation_token,
            ));
            task.run(cancel_fut).await
        };

        self.task_tracker.spawn(run);
    }

    pub async fn spawn_task_loop(&self, task: impl Task + Send + Sync + 'static) {
        let task_name = task.task_name();
        let task_name_copy = task_name.clone();
        let cancellation_token = self.cancellation_token.clone();
        self.task_tracker.spawn(async move {
            let mut shutdown_signal = Box::pin(wait_for_shutdown_signal(
                task_name.clone(),
                cancellation_token,
            ));
            loop {
                select! {
                    // We use biased selection as it would order our futures according to physical
                    // arrangements below
                    // We want shutdown signals to always be checked for first, hence the arrangements
                    biased;

                    _ = &mut shutdown_signal => {
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

    /// Installs custom panic hook and allows app cleanup before dying
    pub fn install_panic_hook(&self) {
        let token = self.cancellation_token();
        let default_hook = std::panic::take_hook();
        std::panic::set_hook(Box::new(move |hook| {
            default_hook(hook);
            log::error!("Thread panicked, shutting down all tasks..");
            token.cancel();
        }));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    struct PanicTask;

    #[async_trait::async_trait]
    impl Task for PanicTask {
        fn task_name(&self) -> String {
            "panic-task".to_string()
        }
        async fn run(&self) -> TaskState {
            panic!("simulated papaya panic: assertion failed: len.is_power_of_two()");
        }
    }

    struct WaitingTask {
        cleaned_up: Arc<AtomicBool>,
    }

    #[async_trait::async_trait]
    impl Task for WaitingTask {
        fn task_name(&self) -> String {
            "waiting-task".to_string()
        }
        async fn run(&self) -> TaskState {
            // Yield back to the runtime repeatedly, simulating a long-running task
            std::future::pending::<()>().await;
            TaskState::Continue
        }
        async fn cleanup(&self) {
            self.cleaned_up.store(true, Ordering::SeqCst);
        }
    }

    #[tokio::test]
    async fn test_panic_triggers_shutdown_of_all_tasks() {
        let manager = TaskManager::new();
        manager.install_panic_hook();

        let cleaned_up = Arc::new(AtomicBool::new(false));

        // Spawn a long-running task that should get cancelled when the panic fires
        manager
            .spawn_task_loop(WaitingTask {
                cleaned_up: cleaned_up.clone(),
            })
            .await;

        // Spawn a task that will panic
        manager.spawn_task_loop(PanicTask).await;

        // wait() should return because the panic hook cancels all tasks.
        // If the hook doesn't work, this will hang forever — the test runner
        // will kill it after its timeout.
        manager.wait().await;

        // The waiting task should have run its cleanup
        assert!(
            cleaned_up.load(Ordering::SeqCst),
            "WaitingTask cleanup should have been called during shutdown"
        );
    }
}
