use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use ahnlich_db::engine::predicate::benchmark::PredicateDeletionBenchmark;
use ahnlich_types::keyval::StoreValue;
use ahnlich_types::metadata::{MetadataValue, metadata_value};
use ahnlich_types::utils::StoreKeyId;
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

#[derive(Clone, Copy)]
struct Scenario {
    name: &'static str,
    entry_count: usize,
    indexed_key_count: usize,
    unindexed_key_count: usize,
    cardinality: usize,
    delete_count: usize,
}

const SCENARIOS: [Scenario; 6] = [
    Scenario {
        name: "low_cardinality_sparse_delete",
        entry_count: 10_000,
        indexed_key_count: 4,
        unindexed_key_count: 0,
        cardinality: 100,
        delete_count: 1,
    },
    Scenario {
        name: "high_cardinality_sparse_delete",
        entry_count: 100_000,
        indexed_key_count: 4,
        unindexed_key_count: 0,
        cardinality: 100_000,
        delete_count: 1,
    },
    Scenario {
        name: "high_cardinality_batch_delete",
        entry_count: 100_000,
        indexed_key_count: 4,
        unindexed_key_count: 0,
        cardinality: 100_000,
        delete_count: 100,
    },
    Scenario {
        name: "low_cardinality_dense_delete",
        entry_count: 100_000,
        indexed_key_count: 4,
        unindexed_key_count: 0,
        cardinality: 100,
        delete_count: 10_000,
    },
    Scenario {
        name: "many_predicate_indexes",
        entry_count: 50_000,
        indexed_key_count: 8,
        unindexed_key_count: 0,
        cardinality: 10_000,
        delete_count: 100,
    },
    Scenario {
        name: "wide_metadata",
        entry_count: 50_000,
        indexed_key_count: 4,
        unindexed_key_count: 12,
        cardinality: 10_000,
        delete_count: 100,
    },
];

fn metadata_value(value_index: usize) -> MetadataValue {
    MetadataValue {
        value: Some(metadata_value::Value::RawString(format!(
            "value-{value_index}"
        ))),
    }
}

fn fixture(scenario: Scenario) -> PredicateDeletionBenchmark {
    let indexed_keys = (0..scenario.indexed_key_count)
        .map(|index| format!("indexed-{index}"))
        .collect::<Vec<_>>();

    let entries = (0..scenario.entry_count)
        .map(|entry_index| {
            let mut metadata =
                HashMap::with_capacity(scenario.indexed_key_count + scenario.unindexed_key_count);

            for key_index in 0..scenario.indexed_key_count {
                metadata.insert(
                    format!("indexed-{key_index}"),
                    metadata_value((entry_index + key_index) % scenario.cardinality),
                );
            }
            for key_index in 0..scenario.unindexed_key_count {
                metadata.insert(
                    format!("unindexed-{key_index}"),
                    metadata_value((entry_index + key_index) % scenario.cardinality),
                );
            }

            (
                StoreKeyId(entry_index as u64),
                Arc::new(StoreValue { value: metadata }),
            )
        })
        .collect::<Vec<_>>();

    let removed = (0..scenario.delete_count)
        .map(|delete_index| {
            let entry_index = delete_index * scenario.entry_count / scenario.delete_count;
            entries[entry_index].clone()
        })
        .collect();

    PredicateDeletionBenchmark::new(indexed_keys, entries, removed)
}

fn assert_equivalent(scenario: Scenario) {
    let control = fixture(scenario);
    control.remove_control();

    let candidate = fixture(scenario);
    candidate.remove_candidate();

    assert_eq!(
        control.snapshot(),
        candidate.snapshot(),
        "control and candidate differed for {}",
        scenario.name
    );
}

fn measure_repeated_deletion(
    benchmark: &PredicateDeletionBenchmark,
    iterations: u64,
    delete: impl Fn(&PredicateDeletionBenchmark),
) -> Duration {
    let mut measured = Duration::ZERO;
    for _ in 0..iterations {
        let start = Instant::now();
        delete(benchmark);
        measured += start.elapsed();

        // Restoration is deliberately excluded from deletion latency.
        benchmark.restore_removed();
    }
    measured
}

fn predicate_index_deletion(c: &mut Criterion) {
    let mut group = c.benchmark_group("predicate_index_deletion");
    group.sampling_mode(criterion::SamplingMode::Flat);

    for scenario in SCENARIOS {
        assert_equivalent(scenario);

        group.bench_function(BenchmarkId::new("control", scenario.name), |b| {
            let benchmark = fixture(scenario);
            b.iter_custom(|iterations| {
                measure_repeated_deletion(&benchmark, iterations, |benchmark| {
                    benchmark.remove_control();
                })
            });
        });

        group.bench_function(BenchmarkId::new("candidate", scenario.name), |b| {
            let benchmark = fixture(scenario);
            b.iter_custom(|iterations| {
                measure_repeated_deletion(&benchmark, iterations, |benchmark| {
                    benchmark.remove_candidate();
                })
            });
        });
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3));
    targets = predicate_index_deletion
}
criterion_main!(benches);
