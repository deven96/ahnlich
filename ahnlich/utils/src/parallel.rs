use rayon::ThreadPoolBuilder;
use std::sync::Once;

static INIT_THREADPOOL_ONCE: Once = Once::new();

// Initialize global rayon threadpool
pub(crate) fn init_threadpool(num_threads: usize) {
    INIT_THREADPOOL_ONCE.call_once(|| {
        ThreadPoolBuilder::new()
            .num_threads(num_threads)
            .build_global()
            .expect("Cannot build server threadpool");
    });
}

// Calculates chunk size to use for an iterable input in order for it to be able to fit into all
// possible rayon threads
pub fn chunk_size(input_length: usize) -> usize {
    let num_threads = rayon::current_num_threads();
    let minimum_factor = std::cmp::min(input_length, num_threads);
    input_length.div_ceil(minimum_factor)
}
