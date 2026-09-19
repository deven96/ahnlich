use std::collections::BTreeSet;
use std::hint::black_box;
use std::ops::Bound::{Excluded, Unbounded};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::{Duration, Instant};

use ahnlich_types::utils::StoreKeyId;
use crossbeam_skiplist::SkipSet;
use parking_lot::RwLock;
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};

type LockedBTreeSet = RwLock<BTreeSet<StoreKeyId>>;

const ENTRY_COUNT: usize = 100_000;
const PAGE_LIMIT: usize = 100;
const TEST_DURATION: Duration = Duration::from_secs(3);
const READER_COUNTS: [usize; 4] = [1, 4, 16, 32];
const WRITER_COUNTS: [usize; 2] = [1, 4];
const MUTATION_IDS_PER_WRITER: usize = 1_024;
const LATENCY_SAMPLE_INTERVAL: u64 = 127;

struct ContentionResult {
    elapsed: Duration,
    reader_operations: u64,
    writer_operations: u64,
    writer_latencies: Vec<Duration>,
}

fn generate_ids(count: usize, seed: u64) -> Vec<StoreKeyId> {
    let mut random = StdRng::seed_from_u64(seed);

    (0..count).map(|_| StoreKeyId(random.next_u64())).collect()
}

fn sorted_ids(ids: &[StoreKeyId]) -> Vec<StoreKeyId> {
    let mut sorted = ids.to_vec();
    sorted.sort_unstable();
    sorted.dedup();
    sorted
}

fn build_btree(ids: &[StoreKeyId]) -> LockedBTreeSet {
    RwLock::new(ids.iter().copied().collect())
}

fn build_skipset(ids: &[StoreKeyId]) -> SkipSet<StoreKeyId> {
    let index = SkipSet::new();

    for id in ids {
        index.insert(*id);
    }

    index
}

fn btree_page(index: &LockedBTreeSet, cursor: Option<StoreKeyId>) -> Vec<StoreKeyId> {
    let index = index.read();

    match cursor {
        Some(cursor) => index
            .range((Excluded(cursor), Unbounded))
            .take(PAGE_LIMIT)
            .copied()
            .collect(),
        None => index.iter().take(PAGE_LIMIT).copied().collect(),
    }
}

fn skipset_page(index: &SkipSet<StoreKeyId>, cursor: Option<StoreKeyId>) -> Vec<StoreKeyId> {
    match cursor {
        Some(cursor) => index
            .range((Excluded(cursor), Unbounded))
            .take(PAGE_LIMIT)
            .map(|entry| *entry.value())
            .collect(),
        None => index
            .iter()
            .take(PAGE_LIMIT)
            .map(|entry| *entry.value())
            .collect(),
    }
}

fn page_checksum(page: &[StoreKeyId]) -> u64 {
    page.iter()
        .fold(0_u64, |checksum, id| checksum.wrapping_add(id.0))
}

fn run_contention<ReadPage, Mutate>(
    reader_count: usize,
    writer_count: usize,
    cursors: &[Option<StoreKeyId>],
    mutation_ids: &[Vec<StoreKeyId>],
    read_page: &ReadPage,
    mutate: &Mutate,
) -> ContentionResult
where
    ReadPage: Fn(Option<StoreKeyId>) -> u64 + Sync,
    Mutate: Fn(StoreKeyId, bool) + Sync,
{
    thread::scope(|scope| {
        let start = Arc::new(Barrier::new(reader_count + writer_count + 1));
        let stop = Arc::new(AtomicBool::new(false));

        let mut reader_handles = Vec::with_capacity(reader_count);

        for reader_number in 0..reader_count {
            let start = Arc::clone(&start);
            let stop = Arc::clone(&stop);

            reader_handles.push(scope.spawn(move || {
                start.wait();

                let mut operations = 0_u64;
                let mut checksum = 0_u64;

                while !stop.load(Ordering::Relaxed) {
                    let cursor = cursors[(operations as usize + reader_number) % cursors.len()];

                    checksum = checksum.wrapping_add(read_page(cursor));
                    operations += 1;
                }

                black_box(checksum);
                operations
            }));
        }

        let mut writer_handles = Vec::with_capacity(writer_count);

        for writer_number in 0..writer_count {
            let start = Arc::clone(&start);
            let stop = Arc::clone(&stop);
            let ids = &mutation_ids[writer_number];

            writer_handles.push(scope.spawn(move || {
                start.wait();

                let mut operations = 0_u64;
                let mut latencies = Vec::new();

                while !stop.load(Ordering::Relaxed) {
                    let id = ids[(operations as usize / 2) % ids.len()];
                    let insert = operations.is_multiple_of(2);
                    let should_sample = operations.is_multiple_of(LATENCY_SAMPLE_INTERVAL);

                    if should_sample {
                        let started = Instant::now();
                        mutate(id, insert);
                        latencies.push(started.elapsed());
                    } else {
                        mutate(id, insert);
                    }

                    operations += 1;
                }

                (operations, latencies)
            }));
        }

        start.wait();

        let started = Instant::now();
        thread::sleep(TEST_DURATION);
        stop.store(true, Ordering::Relaxed);
        let elapsed = started.elapsed();

        let reader_operations = reader_handles
            .into_iter()
            .map(|handle| handle.join().expect("reader should not panic"))
            .sum();

        let mut writer_operations = 0_u64;
        let mut writer_latencies = Vec::new();

        for handle in writer_handles {
            let (operations, mut latencies) = handle.join().expect("writer should not panic");

            writer_operations += operations;
            writer_latencies.append(&mut latencies);
        }

        ContentionResult {
            elapsed,
            reader_operations,
            writer_operations,
            writer_latencies,
        }
    })
}

fn percentile_micros(sorted: &[Duration], percentile: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }

    let index = ((sorted.len() - 1) as f64 * percentile).round() as usize;
    sorted[index].as_secs_f64() * 1_000_000.0
}

fn print_result(
    index_name: &str,
    reader_count: usize,
    writer_count: usize,
    mut result: ContentionResult,
) {
    result.writer_latencies.sort_unstable();

    let seconds = result.elapsed.as_secs_f64();
    let reader_throughput = result.reader_operations as f64 / seconds;
    let writer_throughput = result.writer_operations as f64 / seconds;

    println!(
        "{index_name:<10} {reader_count:>7} {writer_count:>7} \
         {reader_throughput:>14.0} {writer_throughput:>14.0} \
         {:>12.3} {:>12.3} {:>12.3}",
        percentile_micros(&result.writer_latencies, 0.50),
        percentile_micros(&result.writer_latencies, 0.95),
        percentile_micros(&result.writer_latencies, 0.99),
    );
}

fn main() {
    let ids = generate_ids(ENTRY_COUNT, 42);
    let sorted = sorted_ids(&ids);

    let cursors = [
        None,
        Some(sorted[sorted.len() / 4]),
        Some(sorted[sorted.len() / 2]),
        Some(sorted[(sorted.len() * 3) / 4]),
    ];

    println!(
        "{:<10} {:>7} {:>7} {:>14} {:>14} {:>12} {:>12} {:>12}",
        "index",
        "readers",
        "writers",
        "reader ops/s",
        "writer ops/s",
        "writer p50",
        "writer p95",
        "writer p99",
    );

    println!(
        "{:<10} {:>7} {:>7} {:>14} {:>14} {:>12} {:>12} {:>12}",
        "", "", "", "", "", "(µs)", "(µs)", "(µs)",
    );

    for reader_count in READER_COUNTS {
        for writer_count in WRITER_COUNTS {
            let mutation_ids = (0..writer_count)
                .map(|writer_number| {
                    generate_ids(MUTATION_IDS_PER_WRITER, 1_000 + writer_number as u64)
                })
                .collect::<Vec<_>>();

            let btree = build_btree(&ids);

            let btree_result = run_contention(
                reader_count,
                writer_count,
                &cursors,
                &mutation_ids,
                &|cursor| page_checksum(&btree_page(&btree, cursor)),
                &|id, insert| {
                    let mut index = btree.write();

                    if insert {
                        let _ = index.insert(id);
                    } else {
                        let _ = index.remove(&id);
                    }
                },
            );

            print_result("btree", reader_count, writer_count, btree_result);

            let skipset = build_skipset(&ids);

            let skipset_result = run_contention(
                reader_count,
                writer_count,
                &cursors,
                &mutation_ids,
                &|cursor| page_checksum(&skipset_page(&skipset, cursor)),
                &|id, insert| {
                    if insert {
                        let _ = skipset.insert(id);
                    } else {
                        let _ = skipset.remove(&id);
                    }
                },
            );

            print_result("skipset", reader_count, writer_count, skipset_result);
        }
    }
}
