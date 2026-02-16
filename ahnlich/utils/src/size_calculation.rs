use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;
use task_manager::Task;
use task_manager::TaskState;
use tokio::time::Duration;
use tokio::time::sleep;

/// Background task that periodically recalculates store sizes
/// when modifications have occurred (write_flag is true)
#[derive(Debug, Clone)]
pub struct SizeCalculation<T>
where
    T: Clone,
{
    write_flag: Arc<AtomicBool>,
    calculation_interval: u64,
    stores: T,
}

#[async_trait::async_trait]
impl<T> Task for SizeCalculation<T>
where
    T: Clone + Sync + Send + SizeCalculationHandler,
{
    fn task_name(&self) -> String {
        "size_calculation".to_string()
    }

    async fn run(&self) -> TaskState {
        if self.should_recalculate().await {
            log::debug!("Recalculating store sizes due to write activity");
            self.stores.recalculate_all_sizes();
        }
        TaskState::Continue
    }
}

impl<T> SizeCalculation<T>
where
    T: Clone,
{
    pub fn task(write_flag: Arc<AtomicBool>, calculation_interval: u64, stores: T) -> Self {
        Self {
            write_flag,
            calculation_interval,
            stores,
        }
    }

    async fn should_recalculate(&self) -> bool {
        sleep(Duration::from_millis(self.calculation_interval)).await;
        self.write_flag.load(Ordering::Relaxed)
    }
}

/// Trait that must be implemented by store handlers to support background size calculation  
/// The implementation should iterate through stores and recalculate sizes for dirty ones
pub trait SizeCalculationHandler: Clone + Send + Sync + 'static {
    /// Recalculate sizes for all stores
    fn recalculate_all_sizes(&self);
}
