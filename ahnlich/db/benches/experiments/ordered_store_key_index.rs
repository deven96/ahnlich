use std::collections::BTreeSet;
use std::ops::Bound::{Excluded, Unbounded};
use std::sync::{Arc, Barrier};
use std::thread;
use std::time::Duration;

use ahnlich_types::utils::StoreKeyId;
use criterion::{
    BatchSize, BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main,
};
use crossbeam_skiplist::SkipSet;
use parking_lot::RwLock;
use rand::rngs::StdRng;
use rand::{RngCore, SeedableRng};

type LockedBTreeSet = RwLock<BTreeSet<StoreKeyId>>;

const PAGE_ENTRY_COUNTS: [usize; 3] = [10_000, 100_000, 1_000_000];
const MUTATION_ENTRY_COUNTS: [usize; 2] = [10_000, 100_000];
const PAGE_LIMITS: [usize; 2] = [100, 1_000];
const MIXED_READERS: usize = 4;
const MIXED_READS_PER_READER: usize = 100;
const MIXED_WRITES: usize = 1_000;
const MIXED_PAGE_LIMIT: usize = 100;

fn generate_ids(count: usize) -> Vec<StoreKeyId> {
    let mut random = StdRng::seed_from_u64(42);

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

fn btree_page(index: &LockedBTreeSet, cursor: Option<StoreKeyId>, limit: usize) -> Vec<StoreKeyId> {
    let index = index.read();

    match cursor {
        Some(cursor) => index
            .range((Excluded(cursor), Unbounded))
            .take(limit)
            .copied()
            .collect(),
        None => index.iter().take(limit).copied().collect(),
    }
}

fn skipset_page(
    index: &SkipSet<StoreKeyId>,
    cursor: Option<StoreKeyId>,
    limit: usize,
) -> Vec<StoreKeyId> {
    match cursor {
        Some(cursor) => index
            .range((Excluded(cursor), Unbounded))
            .take(limit)
            .map(|entry| *entry.value())
            .collect(),
        None => index
            .iter()
            .take(limit)
            .map(|entry| *entry.value())
            .collect(),
    }
}

fn benchmark_page_reads(criterion: &mut Criterion) {
    for entry_count in PAGE_ENTRY_COUNTS {
        let ids = generate_ids(entry_count);
        let sorted = sorted_ids(&ids);
        let btree = build_btree(&ids);
        let skipset = build_skipset(&ids);

        let cursor_positions = [
            ("start", None),
            ("middle", Some(sorted[sorted.len() / 2])),
            ("late", Some(sorted[(sorted.len() * 3) / 4])),
        ];

        for limit in PAGE_LIMITS {
            let mut group =
                criterion.benchmark_group(format!("ordered_page/{entry_count}/limit_{limit}"));

            group.throughput(Throughput::Elements(limit as u64));

            for (position, cursor) in cursor_positions {
                group.bench_with_input(
                    BenchmarkId::new("btree", position),
                    &cursor,
                    |benchmark, cursor| {
                        benchmark.iter(|| black_box(btree_page(&btree, black_box(*cursor), limit)));
                    },
                );

                group.bench_with_input(
                    BenchmarkId::new("skipset", position),
                    &cursor,
                    |benchmark, cursor| {
                        benchmark
                            .iter(|| black_box(skipset_page(&skipset, black_box(*cursor), limit)));
                    },
                );
            }

            group.finish();
        }
    }
}

fn benchmark_insertions(criterion: &mut Criterion) {
    for entry_count in MUTATION_ENTRY_COUNTS {
        let ids = generate_ids(entry_count);
        let mut group = criterion.benchmark_group(format!("ordered_insert/{entry_count}"));

        group.throughput(Throughput::Elements(entry_count as u64));

        group.bench_function("btree", |benchmark| {
            benchmark.iter_batched(
                || RwLock::new(BTreeSet::new()),
                |index| {
                    index.write().extend(ids.iter().copied());
                    black_box(index);
                },
                BatchSize::LargeInput,
            );
        });

        group.bench_function("skipset", |benchmark| {
            benchmark.iter_batched(
                SkipSet::new,
                |index| {
                    for id in &ids {
                        index.insert(*id);
                    }
                    black_box(index);
                },
                BatchSize::LargeInput,
            );
        });

        group.finish();
    }
}

fn benchmark_deletions(criterion: &mut Criterion) {
    for entry_count in MUTATION_ENTRY_COUNTS {
        let ids = generate_ids(entry_count);
        let mut group = criterion.benchmark_group(format!("ordered_delete/{entry_count}"));

        group.throughput(Throughput::Elements(entry_count as u64));

        group.bench_function("btree", |benchmark| {
            benchmark.iter_batched(
                || build_btree(&ids),
                |index| {
                    let mut index = index.write();

                    for id in &ids {
                        index.remove(id);
                    }

                    black_box(index.len());
                },
                BatchSize::LargeInput,
            );
        });

        group.bench_function("skipset", |benchmark| {
            benchmark.iter_batched(
                || build_skipset(&ids),
                |index| {
                    for id in &ids {
                        index.remove(id);
                    }

                    black_box(index.len());
                },
                BatchSize::LargeInput,
            );
        });

        group.finish();
    }
}

fn run_btree_mixed(
    index: &LockedBTreeSet,
    cursors: &[Option<StoreKeyId>],
    write_ids: &[StoreKeyId],
) -> u64 {
    thread::scope(|scope| {
        let start = Arc::new(Barrier::new(MIXED_READERS + 1));
        let mut readers = Vec::with_capacity(MIXED_READERS);

        for reader_number in 0..MIXED_READERS {
            let start = Arc::clone(&start);

            readers.push(scope.spawn(move || {
                start.wait();

                let mut checksum = 0_u64;

                for operation in 0..MIXED_READS_PER_READER {
                    let cursor = cursors[(operation + reader_number) % cursors.len()];
                    let page = btree_page(index, cursor, MIXED_PAGE_LIMIT);

                    checksum = page.iter().fold(checksum, |sum, id| sum.wrapping_add(id.0));
                }

                checksum
            }));
        }

        let writer_start = Arc::clone(&start);

        let writer = scope.spawn(move || {
            writer_start.wait();

            let mut index = index.write();

            for id in write_ids {
                index.insert(*id);
            }

            for id in write_ids {
                index.remove(id);
            }
        });

        let checksum = readers
            .into_iter()
            .map(|reader| reader.join().expect("reader should not panic"))
            .fold(0_u64, u64::wrapping_add);

        writer.join().expect("writer should not panic");

        checksum
    })
}

fn run_skipset_mixed(
    index: &SkipSet<StoreKeyId>,
    cursors: &[Option<StoreKeyId>],
    write_ids: &[StoreKeyId],
) -> u64 {
    thread::scope(|scope| {
        let start = Arc::new(Barrier::new(MIXED_READERS + 1));
        let mut readers = Vec::with_capacity(MIXED_READERS);

        for reader_number in 0..MIXED_READERS {
            let start = Arc::clone(&start);

            readers.push(scope.spawn(move || {
                start.wait();

                let mut checksum = 0_u64;

                for operation in 0..MIXED_READS_PER_READER {
                    let cursor = cursors[(operation + reader_number) % cursors.len()];
                    let page = skipset_page(index, cursor, MIXED_PAGE_LIMIT);

                    checksum = page.iter().fold(checksum, |sum, id| sum.wrapping_add(id.0));
                }

                checksum
            }));
        }

        let writer_start = Arc::clone(&start);

        let writer = scope.spawn(move || {
            writer_start.wait();

            for id in write_ids {
                index.insert(*id);
            }

            for id in write_ids {
                index.remove(id);
            }
        });

        let checksum = readers
            .into_iter()
            .map(|reader| reader.join().expect("reader should not panic"))
            .fold(0_u64, u64::wrapping_add);

        writer.join().expect("writer should not panic");

        checksum
    })
}

fn benchmark_mixed_workload(criterion: &mut Criterion) {
    let ids = generate_ids(100_000);
    let sorted = sorted_ids(&ids);

    let cursors = [
        None,
        Some(sorted[sorted.len() / 4]),
        Some(sorted[sorted.len() / 2]),
        Some(sorted[(sorted.len() * 3) / 4]),
    ];

    let write_ids = (0..MIXED_WRITES)
        .map(|offset| StoreKeyId(u64::MAX - offset as u64))
        .collect::<Vec<_>>();

    let mut group = criterion.benchmark_group("ordered_mixed/100000");

    group.bench_function("btree", |benchmark| {
        benchmark.iter_batched(
            || build_btree(&ids),
            |index| {
                black_box(run_btree_mixed(&index, &cursors, &write_ids));
            },
            BatchSize::LargeInput,
        );
    });

    group.bench_function("skipset", |benchmark| {
        benchmark.iter_batched(
            || build_skipset(&ids),
            |index| {
                black_box(run_skipset_mixed(&index, &cursors, &write_ids));
            },
            BatchSize::LargeInput,
        );
    });

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(20)
        .warm_up_time(Duration::from_secs(2))
        .measurement_time(Duration::from_secs(5));
    targets =
        benchmark_page_reads,
        benchmark_insertions,
        benchmark_deletions,
        benchmark_mixed_workload
}

criterion_main!(benches);
