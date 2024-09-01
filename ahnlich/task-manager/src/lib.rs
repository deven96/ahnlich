use log::{debug, info};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::{Context, Poll};
use tokio::{select, signal};
/// TaskManager is a struct to achieve multiple things
///
/// - Spawn multiple long running "tasks" that expect to be run in a loop
/// - Break the task loop when one of the following happens:
///     - SIGterm or SIGint is received
///     - External cancellation token is triggered
///     - Task chooses to end via some internal condition
///
/// TaskManager extends the tokio utils TaskTracker to ensure that ever task loop is being tracked
/// and the tasks have the ability to perform any necessary cleanup before ending
use tokio_util::sync::CancellationToken;
use tokio_util::task::TaskTracker;

pub struct CloneableAsyncClosure<F, Fut>
where
    F: Fn() -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = TaskLoopControl> + Send + 'static,
{
    inner: Arc<F>,
    future: Pin<Box<Fut>>,
}

impl<F, Fut> CloneableAsyncClosure<F, Fut>
where
    F: Fn() -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = TaskLoopControl> + Send + 'static,
{
    pub fn new(f: F) -> Self {
        let future = (f)();
        CloneableAsyncClosure {
            inner: Arc::new(f),
            future: Box::pin(future),
        }
    }

    fn reset(&mut self) {
        self.future = Box::pin((self.inner)());
    }
}

impl<F, Fut> Future for CloneableAsyncClosure<F, Fut>
where
    F: Fn() -> Fut + Clone + Send + Sync + 'static,
    Fut: Future<Output = TaskLoopControl> + Send + 'static,
{
    type Output = TaskLoopControl;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.future.as_mut().poll(cx)
    }
}

#[derive(Debug, Clone)]
pub enum TaskLoopControl {
    Break,
    Continue,
}

#[derive(Debug, Clone)]
pub struct TaskManager {
    cancellation_token: CancellationToken,
    task_tracker: TaskTracker,
}

impl TaskManager {
    pub fn new() -> Self {
        Self {
            cancellation_token: CancellationToken::new(),
            task_tracker: TaskTracker::new(),
        }
    }

    pub async fn spawn_task_loop<F>(&self, task: F, task_name: String)
    where
        F: Future<Output = TaskLoopControl> + Send + 'static,
    {
        let cancellation_token = self.cancellation_token.clone();
        let task_name_copy = task_name.clone();
        let mut task = Box::pin(task);

        self.task_tracker.spawn(async move {
            loop {
                let loop_control = select! {
                    // We use biased selection as it would order our futures according to physical
                    // arrangements below
                    // We want shutdown signals to always be checked for first, hence the arrangements
                    biased;

                    _ = signal::ctrl_c() => {
                        info!("Received Ctrl-C signal, cancelling [{task_name}] task");
                        TaskLoopControl::Break
                    }
                    _ = cancellation_token.cancelled() => {
                        info!("Received Cancellation token signal, cancelling [{task_name}] task");
                        TaskLoopControl::Break
                    }
                    result = &mut task => {
                        result
                    }
                };
                match loop_control {
                    TaskLoopControl::Continue => {
                        debug!("Continuing [{task_name}] task loop");
                        continue;
                    }
                    TaskLoopControl::Break => {
                        debug!("[{task_name}] Task cancelled");
                        break;
                    }
                }
            }
        });

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
}

#[cfg(test)]
mod tests {
    use super::*;
}
