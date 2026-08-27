use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use ahnlich_db::engine::store::ParallelismConfig;
use ahnlich_similarity::heap::BoundedMaxHeap;
use ahnlich_similarity::{Closeness, DistanceFn, EmbeddingKey, LinearAlgorithm};
use ahnlich_types::keyval::StoreValue;
use ahnlich_types::utils::StoreKeyId;
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};
use papaya::HashMap;
use rayon::ThreadPoolBuilder;
use rayon::iter::{ParallelBridge, ParallelIterator};
use rayon::prelude::*;

type StoreMap = HashMap<StoreKeyId, (EmbeddingKey, Arc<StoreValue>)>;

const ENTRY_COUNTS: [usize; 3] = [1_000, 10_000, 100_000];
const DIMENSIONS: [usize; 2] = [128, 768];
const RAYON_THREAD_COUNTS: [usize; 5] = [1, 2, 4, 8, 10];
const POLICY_ENTRY_COUNTS: [usize; 7] = [500, 1_000, 2_500, 5_000, 10_000, 25_000, 100_000];
const POLICY_DIMENSIONS: [usize; 5] = [64, 128, 384, 768, 1_536];
const CROSSOVER_DIMENSIONS: [usize; 4] = [256, 512, 1_024, 2_048];
const FULL_DIMENSION_MATRIX: [usize; 9] = [64, 128, 256, 384, 512, 768, 1_024, 1_536, 2_048];
const CONCURRENT_DIMENSIONS: [usize; 4] = [128, 512, 1_024, 2_048];
const CONCURRENT_WORK_MULTIPLIERS: [usize; 3] = [1, 2, 4];
const CONCURRENCY_LEVELS: [usize; 6] = [1, 2, 4, 8, 16, 32];
const MINIMUM_WORK_THRESHOLDS: [usize; 5] = [256_000, 384_000, 512_000, 768_000, 1_000_000];
const ACTIVE_REQUESTS: usize = 1;
const TOP_N: usize = 50;

#[derive(Debug)]
struct Candidate {
    key_id: StoreKeyId,
    closeness: Closeness,
    score: f32,
}

impl PartialEq for Candidate {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == std::cmp::Ordering::Equal
    }
}

impl Eq for Candidate {}

impl PartialOrd for Candidate {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for Candidate {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.closeness
            .cmp(&other.closeness)
            .then_with(|| self.key_id.0.cmp(&other.key_id.0))
    }
}

fn find_top_n_parallel<'a>(
    algorithm: LinearAlgorithm,
    search_vector: &EmbeddingKey,
    search_list: impl ParallelIterator<Item = (&'a StoreKeyId, &'a EmbeddingKey, &'a StoreValue)>,
) -> Vec<(StoreKeyId, f32)> {
    let capacity = NonZeroUsize::new(TOP_N).expect("TOP_N must be non-zero");

    search_list
        .fold(
            || BoundedMaxHeap::new(capacity),
            |mut heap, (key_id, vector, _store_value)| {
                let score = algorithm.score(search_vector.as_slice(), vector.as_slice());
                heap.push(Candidate {
                    key_id: *key_id,
                    closeness: score.closeness(),
                    score: score.value(),
                });
                heap
            },
        )
        .reduce(
            || BoundedMaxHeap::new(capacity),
            |mut left, right| {
                for item in right.into_sorted_vec() {
                    left.push(item);
                }
                left
            },
        )
        .into_sorted_vec()
        .into_iter()
        .map(|candidate| (candidate.key_id, candidate.score))
        .collect()
}

fn find_top_n_sequential<'a>(
    algorithm: LinearAlgorithm,
    search_vector: &EmbeddingKey,
    search_list: impl Iterator<Item = (&'a StoreKeyId, &'a EmbeddingKey, &'a StoreValue)>,
) -> Vec<(StoreKeyId, f32)> {
    let capacity = NonZeroUsize::new(TOP_N).expect("TOP_N must be non-zero");
    let mut heap = BoundedMaxHeap::new(capacity);

    for (key_id, vector, _store_value) in search_list {
        let score = algorithm.score(search_vector.as_slice(), vector.as_slice());
        heap.push(Candidate {
            key_id: *key_id,
            closeness: score.closeness(),
            score: score.value(),
        });
    }

    heap.into_sorted_vec()
        .into_iter()
        .map(|candidate| (candidate.key_id, candidate.score))
        .collect()
}

fn sequential_direct(entries: &StoreMap, search_vector: &EmbeddingKey) -> Vec<(StoreKeyId, f32)> {
    let pinned = entries.pin();

    find_top_n_sequential(
        LinearAlgorithm::EuclideanDistance,
        search_vector,
        pinned
            .into_iter()
            .map(|(id, (key, value))| (id, key, value.as_ref())),
    )
}

fn current_cloned_vec(entries: &StoreMap, search_vector: &EmbeddingKey) -> Vec<(StoreKeyId, f32)> {
    let pinned = entries.pin();
    let search_list: Vec<_> = pinned
        .into_iter()
        .map(|(id, (key, value))| (*id, key.clone(), Arc::clone(value)))
        .collect();

    find_top_n_parallel(
        LinearAlgorithm::EuclideanDistance,
        search_vector,
        search_list
            .par_iter()
            .map(|(id, key, value)| (id, key, value.as_ref())),
    )
}

fn borrowed_vec(entries: &StoreMap, search_vector: &EmbeddingKey) -> Vec<(StoreKeyId, f32)> {
    let pinned = entries.pin_owned();
    let search_list: Vec<_> = pinned.iter().collect();

    find_top_n_parallel(
        LinearAlgorithm::EuclideanDistance,
        search_vector,
        search_list.par_iter().map(|entry| {
            let (id, (key, value)) = *entry;
            (id, key, value.as_ref())
        }),
    )
}

fn streamed(entries: &StoreMap, search_vector: &EmbeddingKey) -> Vec<(StoreKeyId, f32)> {
    let pinned = entries.pin_owned();

    find_top_n_parallel(
        LinearAlgorithm::EuclideanDistance,
        search_vector,
        pinned
            .iter()
            .par_bridge()
            .map(|(id, (key, value))| (id, key, value.as_ref())),
    )
}

fn execute_search(
    use_parallel: bool,
    entries: &StoreMap,
    search_vector: &EmbeddingKey,
) -> Vec<(StoreKeyId, f32)> {
    if use_parallel {
        borrowed_vec(entries, search_vector)
    } else {
        sequential_direct(entries, search_vector)
    }
}

fn execute_search_with_staging(
    use_parallel: bool,
    use_borrowed_staging: bool,
    entries: &StoreMap,
    search_vector: &EmbeddingKey,
) -> Vec<(StoreKeyId, f32)> {
    match (use_parallel, use_borrowed_staging) {
        (false, _) => sequential_direct(entries, search_vector),
        (true, false) => current_cloned_vec(entries, search_vector),
        (true, true) => borrowed_vec(entries, search_vector),
    }
}

const fn cloned_staging_dimension_band_entry_threshold(dimensions: usize) -> usize {
    match dimensions {
        0..=128 => 100_000,
        129..=256 => 25_000,
        257..=512 => 10_000,
        513..=768 => 5_000,
        769..=1_024 => 2_500,
        1_025..=1_536 => 1_000,
        _ => 500,
    }
}

fn should_parallelize_dimension_band(
    entry_count: usize,
    dimensions: usize,
    active_requests: usize,
    num_threads: usize,
    high_concurrency_threshold: usize,
) -> bool {
    let base_threshold = cloned_staging_dimension_band_entry_threshold(dimensions);
    let concurrency_factor = if active_requests < high_concurrency_threshold {
        1
    } else {
        (active_requests / num_threads).max(1)
    };

    entry_count >= base_threshold.saturating_mul(concurrency_factor)
}

const fn borrowed_staging_dimension_band_entry_threshold(dimensions: usize) -> usize {
    match dimensions {
        0..=64 => 100_000,
        65..=128 => 25_000,
        129..=384 => 10_000,
        385..=512 => 5_000,
        513..=768 => 2_500,
        769..=1_024 => 1_000,
        _ => 500,
    }
}

#[derive(Clone, Copy)]
enum ConcurrentSearchMode {
    Sequential,
    BorrowedRayon,
}

impl ConcurrentSearchMode {
    const fn label(self) -> &'static str {
        match self {
            Self::Sequential => "sequential",
            Self::BorrowedRayon => "borrowed_rayon",
        }
    }

    fn search(self, entries: &StoreMap, search_vector: &EmbeddingKey) {
        let result = match self {
            Self::Sequential => sequential_direct(entries, search_vector),
            Self::BorrowedRayon => borrowed_vec(entries, search_vector),
        };
        black_box(result);
    }
}

struct ConcurrentSearchRunner {
    start: Arc<Barrier>,
    finish: Arc<Barrier>,
    stop: Arc<AtomicBool>,
    workers: Vec<JoinHandle<Vec<Duration>>>,
}

impl ConcurrentSearchRunner {
    fn new(
        concurrency: usize,
        mode: ConcurrentSearchMode,
        entries: Arc<StoreMap>,
        search_vector: Arc<EmbeddingKey>,
    ) -> Self {
        let start = Arc::new(Barrier::new(concurrency + 1));
        let finish = Arc::new(Barrier::new(concurrency + 1));
        let stop = Arc::new(AtomicBool::new(false));
        let workers = (0..concurrency)
            .map(|_| {
                let start = Arc::clone(&start);
                let finish = Arc::clone(&finish);
                let stop = Arc::clone(&stop);
                let entries = Arc::clone(&entries);
                let search_vector = Arc::clone(&search_vector);

                std::thread::spawn(move || {
                    let mut latencies = Vec::new();
                    loop {
                        start.wait();
                        if stop.load(Ordering::Acquire) {
                            break;
                        }

                        let started = Instant::now();
                        mode.search(&entries, &search_vector);
                        latencies.push(started.elapsed());
                        finish.wait();
                    }
                    latencies
                })
            })
            .collect();

        Self {
            start,
            finish,
            stop,
            workers,
        }
    }

    fn run_batch(&self) {
        self.start.wait();
        self.finish.wait();
    }

    fn finish(mut self) -> Vec<Duration> {
        self.stop.store(true, Ordering::Release);
        self.start.wait();
        self.workers
            .drain(..)
            .flat_map(|worker| worker.join().expect("concurrent benchmark worker panicked"))
            .collect()
    }
}

fn percentile(latencies: &mut [Duration], percentile: usize) -> Duration {
    latencies.sort_unstable();
    let index = (latencies.len() - 1).saturating_mul(percentile) / 100;
    latencies[index]
}

fn should_parallelize_dimension_aware(
    entry_count: usize,
    dimensions: usize,
    active_requests: usize,
    num_threads: usize,
    high_concurrency_threshold: usize,
    minimum_work: usize,
) -> bool {
    let estimated_work = entry_count.saturating_mul(dimensions);
    let concurrency_factor = if active_requests < high_concurrency_threshold {
        1
    } else {
        (active_requests / num_threads).max(1)
    };
    let required_work = minimum_work.saturating_mul(concurrency_factor);

    estimated_work >= required_work
}

fn fixture(entry_count: usize, dimensions: usize) -> (StoreMap, EmbeddingKey) {
    fastrand::seed((entry_count as u64) ^ ((dimensions as u64) << 32));

    let entries = StoreMap::with_capacity(entry_count);
    let guard = entries.guard();
    for id in 0..entry_count {
        let embedding = EmbeddingKey::new((0..dimensions).map(|_| fastrand::f32()).collect());
        entries.insert(
            StoreKeyId(id as u64),
            (embedding, Arc::new(StoreValue::default())),
            &guard,
        );
    }
    drop(guard);

    let search_vector = EmbeddingKey::new((0..dimensions).map(|_| fastrand::f32()).collect());
    (entries, search_vector)
}

fn assert_equivalent(entries: &StoreMap, search_vector: &EmbeddingKey) {
    let current = current_cloned_vec(entries, search_vector);
    assert_eq!(sequential_direct(entries, search_vector), current);
    assert_eq!(borrowed_vec(entries, search_vector), current);
    assert_eq!(streamed(entries, search_vector), current);
}

fn papaya_linear_search(c: &mut Criterion) {
    for dimensions in DIMENSIONS {
        let mut group = c.benchmark_group(format!("papaya_linear_search/{dimensions}_dimensions"));
        group.throughput(Throughput::Elements(1));

        for entry_count in ENTRY_COUNTS {
            let (entries, search_vector) = fixture(entry_count, dimensions);
            assert_equivalent(&entries, &search_vector);

            group.throughput(Throughput::Elements(entry_count as u64));
            group.bench_with_input(
                BenchmarkId::new("sequential_direct", entry_count),
                &entry_count,
                |b, _| b.iter(|| sequential_direct(black_box(&entries), black_box(&search_vector))),
            );
            group.bench_with_input(
                BenchmarkId::new("current_cloned_vec", entry_count),
                &entry_count,
                |b, _| {
                    b.iter(|| current_cloned_vec(black_box(&entries), black_box(&search_vector)))
                },
            );
            group.bench_with_input(
                BenchmarkId::new("borrowed_vec", entry_count),
                &entry_count,
                |b, _| b.iter(|| borrowed_vec(black_box(&entries), black_box(&search_vector))),
            );
            group.bench_with_input(
                BenchmarkId::new("streamed_par_bridge", entry_count),
                &entry_count,
                |b, _| b.iter(|| streamed(black_box(&entries), black_box(&search_vector))),
            );
        }

        group.finish();
    }
}

fn rayon_scaling(c: &mut Criterion) {
    for dimensions in DIMENSIONS {
        let mut group = c.benchmark_group(format!("rayon_scaling/{dimensions}_dimensions"));
        group
            .sample_size(10)
            .warm_up_time(Duration::from_millis(500))
            .measurement_time(Duration::from_secs(1));

        for entry_count in ENTRY_COUNTS {
            let (entries, search_vector) = fixture(entry_count, dimensions);
            assert_equivalent(&entries, &search_vector);
            group.throughput(Throughput::Elements(entry_count as u64));
            group.bench_function(
                BenchmarkId::new("sequential_direct", format!("{entry_count}_entries")),
                |b| b.iter(|| sequential_direct(black_box(&entries), black_box(&search_vector))),
            );

            for thread_count in RAYON_THREAD_COUNTS {
                let pool = ThreadPoolBuilder::new()
                    .num_threads(thread_count)
                    .build()
                    .expect("fixed Rayon pool should build");

                let parameter = format!("{entry_count}_entries/{thread_count}_threads");
                group.bench_function(BenchmarkId::new("current_cloned_vec", &parameter), |b| {
                    b.iter(|| {
                        pool.install(|| {
                            current_cloned_vec(black_box(&entries), black_box(&search_vector))
                        })
                    })
                });
                group.bench_function(BenchmarkId::new("borrowed_vec", &parameter), |b| {
                    b.iter(|| {
                        pool.install(|| {
                            borrowed_vec(black_box(&entries), black_box(&search_vector))
                        })
                    })
                });
                group.bench_function(BenchmarkId::new("streamed_par_bridge", &parameter), |b| {
                    b.iter(|| {
                        pool.install(|| streamed(black_box(&entries), black_box(&search_vector)))
                    })
                });
            }
        }

        group.finish();
    }
}

fn adaptive_policy(c: &mut Criterion) {
    let num_threads = rayon::current_num_threads();
    let current_policy = ParallelismConfig::from_cli(num_threads, None, 10_000);
    let mut group = c.benchmark_group("adaptive_policy");
    group
        .sample_size(10)
        .warm_up_time(Duration::from_millis(200))
        .measurement_time(Duration::from_millis(500));

    for dimensions in POLICY_DIMENSIONS {
        for entry_count in POLICY_ENTRY_COUNTS {
            let (entries, search_vector) = fixture(entry_count, dimensions);
            assert_equivalent(&entries, &search_vector);
            group.throughput(Throughput::Elements(entry_count as u64));
            let fixture_name = format!("{entry_count}_entries/{dimensions}_dimensions");

            group.bench_function(
                BenchmarkId::new("baseline_sequential", &fixture_name),
                |b| b.iter(|| sequential_direct(black_box(&entries), black_box(&search_vector))),
            );
            group.bench_function(
                BenchmarkId::new("baseline_borrowed_rayon", &fixture_name),
                |b| b.iter(|| borrowed_vec(black_box(&entries), black_box(&search_vector))),
            );

            let current_uses_parallel =
                current_policy.should_use_parallel(entry_count, ACTIVE_REQUESTS);
            group.bench_function(BenchmarkId::new("current", &fixture_name), |b| {
                b.iter(|| {
                    execute_search(
                        black_box(current_uses_parallel),
                        black_box(&entries),
                        black_box(&search_vector),
                    )
                })
            });

            for minimum_work in MINIMUM_WORK_THRESHOLDS {
                let candidate_uses_parallel = should_parallelize_dimension_aware(
                    entry_count,
                    dimensions,
                    ACTIVE_REQUESTS,
                    num_threads,
                    num_threads,
                    minimum_work,
                );
                group.bench_function(
                    BenchmarkId::new(format!("dimension_aware_{minimum_work}"), &fixture_name),
                    |b| {
                        b.iter(|| {
                            execute_search(
                                black_box(candidate_uses_parallel),
                                black_box(&entries),
                                black_box(&search_vector),
                            )
                        })
                    },
                );
            }
        }
    }

    group.finish();
}

fn dimension_crossover(c: &mut Criterion) {
    let mut group = c.benchmark_group("dimension_crossover");
    group
        .sample_size(10)
        .warm_up_time(Duration::from_millis(200))
        .measurement_time(Duration::from_millis(500));

    for dimensions in CROSSOVER_DIMENSIONS {
        for entry_count in POLICY_ENTRY_COUNTS {
            let (entries, search_vector) = fixture(entry_count, dimensions);
            assert_equivalent(&entries, &search_vector);
            group.throughput(Throughput::Elements(entry_count as u64));
            let fixture_name = format!("{entry_count}_entries/{dimensions}_dimensions");

            group.bench_function(BenchmarkId::new("sequential_direct", &fixture_name), |b| {
                b.iter(|| sequential_direct(black_box(&entries), black_box(&search_vector)))
            });
            group.bench_function(BenchmarkId::new("borrowed_rayon", &fixture_name), |b| {
                b.iter(|| borrowed_vec(black_box(&entries), black_box(&search_vector)))
            });
        }
    }

    group.finish();
}

fn cloned_staging_dimension_crossover(c: &mut Criterion) {
    let mut group = c.benchmark_group("cloned_staging_dimension_crossover");
    group
        .sample_size(20)
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(1));

    for dimensions in FULL_DIMENSION_MATRIX {
        for entry_count in POLICY_ENTRY_COUNTS {
            let (entries, search_vector) = fixture(entry_count, dimensions);
            assert_equivalent(&entries, &search_vector);
            group.throughput(Throughput::Elements(entry_count as u64));
            let fixture_name = format!("{entry_count}_entries/{dimensions}_dimensions");

            group.bench_function(BenchmarkId::new("sequential_direct", &fixture_name), |b| {
                b.iter(|| sequential_direct(black_box(&entries), black_box(&search_vector)))
            });
            group.bench_function(
                BenchmarkId::new("cloned_staging_rayon", &fixture_name),
                |b| b.iter(|| current_cloned_vec(black_box(&entries), black_box(&search_vector))),
            );
        }
    }

    group.finish();
}

fn optimization_ab(c: &mut Criterion) {
    let num_threads = rayon::current_num_threads();
    let current_policy = ParallelismConfig::from_cli(num_threads, None, 10_000);
    let mut group = c.benchmark_group("optimization_ab");
    group
        .sample_size(10)
        .warm_up_time(Duration::from_millis(200))
        .measurement_time(Duration::from_millis(500));

    for dimensions in FULL_DIMENSION_MATRIX {
        for entry_count in POLICY_ENTRY_COUNTS {
            let (entries, search_vector) = fixture(entry_count, dimensions);
            assert_equivalent(&entries, &search_vector);
            group.throughput(Throughput::Elements(entry_count as u64));
            let fixture_name = format!("{entry_count}_entries/{dimensions}_dimensions");
            let current_uses_parallel =
                current_policy.should_use_parallel(entry_count, ACTIVE_REQUESTS);
            let bands_use_parallel = should_parallelize_dimension_band(
                entry_count,
                dimensions,
                ACTIVE_REQUESTS,
                num_threads,
                num_threads,
            );

            group.bench_function(BenchmarkId::new("control", &fixture_name), |b| {
                b.iter(|| {
                    execute_search_with_staging(
                        black_box(current_uses_parallel),
                        black_box(false),
                        black_box(&entries),
                        black_box(&search_vector),
                    )
                })
            });
            group.bench_function(BenchmarkId::new("borrowed_vec_only", &fixture_name), |b| {
                b.iter(|| {
                    execute_search_with_staging(
                        black_box(current_uses_parallel),
                        black_box(true),
                        black_box(&entries),
                        black_box(&search_vector),
                    )
                })
            });
            group.bench_function(
                BenchmarkId::new("dimension_bands_only", &fixture_name),
                |b| {
                    b.iter(|| {
                        execute_search_with_staging(
                            black_box(bands_use_parallel),
                            black_box(false),
                            black_box(&entries),
                            black_box(&search_vector),
                        )
                    })
                },
            );
            group.bench_function(BenchmarkId::new("candidate", &fixture_name), |b| {
                b.iter(|| {
                    execute_search_with_staging(
                        black_box(bands_use_parallel),
                        black_box(true),
                        black_box(&entries),
                        black_box(&search_vector),
                    )
                })
            });
        }
    }

    group.finish();
}

fn concurrent_borrowed_policy(c: &mut Criterion) {
    let rayon_threads = rayon::current_num_threads();
    let mut group = c.benchmark_group("concurrent_borrowed_policy");
    group
        .sample_size(10)
        .warm_up_time(Duration::from_millis(500))
        .measurement_time(Duration::from_secs(1));

    println!("CONCURRENT_CONFIG rayon_threads={rayon_threads}");

    for dimensions in CONCURRENT_DIMENSIONS {
        let base_threshold = borrowed_staging_dimension_band_entry_threshold(dimensions);
        for work_multiplier in CONCURRENT_WORK_MULTIPLIERS {
            let entry_count = base_threshold.saturating_mul(work_multiplier);
            let (entries, search_vector) = fixture(entry_count, dimensions);
            assert_equivalent(&entries, &search_vector);
            let entries = Arc::new(entries);
            let search_vector = Arc::new(search_vector);

            for concurrency in CONCURRENCY_LEVELS {
                let current_concurrency_factor = if concurrency < rayon_threads {
                    1
                } else {
                    (concurrency / rayon_threads).max(1)
                };
                let current_threshold = base_threshold.saturating_mul(current_concurrency_factor);
                let linear_threshold = base_threshold.saturating_mul(concurrency.max(1));
                let current_uses_parallel = entry_count >= current_threshold;
                let linear_uses_parallel = entry_count >= linear_threshold;
                println!(
                    "CONCURRENT_POLICY entries={entry_count} dimensions={dimensions} \
                     work_multiplier={work_multiplier} concurrency={concurrency} \
                     base_threshold={base_threshold} current_threshold={current_threshold} \
                     current_path={} linear_threshold={linear_threshold} linear_path={}",
                    if current_uses_parallel {
                        "borrowed_rayon"
                    } else {
                        "sequential"
                    },
                    if linear_uses_parallel {
                        "borrowed_rayon"
                    } else {
                        "sequential"
                    }
                );

                group.throughput(Throughput::Elements(concurrency as u64));
                let fixture_name =
                    format!("{entry_count}_entries_{dimensions}_dimensions_c{concurrency}");

                for mode in [
                    ConcurrentSearchMode::Sequential,
                    ConcurrentSearchMode::BorrowedRayon,
                ] {
                    let runner = ConcurrentSearchRunner::new(
                        concurrency,
                        mode,
                        Arc::clone(&entries),
                        Arc::clone(&search_vector),
                    );
                    group.bench_function(BenchmarkId::new(mode.label(), &fixture_name), |b| {
                        b.iter(|| runner.run_batch())
                    });

                    let mut latencies = runner.finish();
                    let samples = latencies.len();
                    let p50 = percentile(&mut latencies, 50);
                    let p95 = percentile(&mut latencies, 95);
                    let p99 = percentile(&mut latencies, 99);
                    println!(
                        "CONCURRENT_LATENCY path={} entries={entry_count} \
                         dimensions={dimensions} work_multiplier={work_multiplier} \
                         concurrency={concurrency} samples={samples} p50_ns={} \
                         p95_ns={} p99_ns={}",
                        mode.label(),
                        p50.as_nanos(),
                        p95.as_nanos(),
                        p99.as_nanos(),
                    );
                }
            }
        }
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(20)
        .warm_up_time(Duration::from_secs(2))
        .measurement_time(Duration::from_secs(5));
    targets = papaya_linear_search, rayon_scaling, adaptive_policy, dimension_crossover, cloned_staging_dimension_crossover, optimization_ab, concurrent_borrowed_policy
}
criterion_main!(benches);
