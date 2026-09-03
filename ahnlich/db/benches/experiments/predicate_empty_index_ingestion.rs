use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use ahnlich_db::engine::predicate::benchmark::EmptyPredicateIngestionBenchmark;
use ahnlich_types::keyval::StoreValue;
use ahnlich_types::metadata::{MetadataValue, metadata_value};
use ahnlich_types::utils::StoreKeyId;
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};

const BATCH_SIZES: [usize; 4] = [100, 1_000, 10_000, 100_000];
const METADATA_FIELD_COUNTS: [usize; 3] = [0, 4, 16];

#[derive(Clone, Copy)]
enum MetadataFilteringPolicy {
    Sequential,
    Adaptive,
}

impl MetadataFilteringPolicy {
    const fn label(self) -> &'static str {
        match self {
            Self::Sequential => "sequential_filtering",
            Self::Adaptive => "adaptive_filtering",
        }
    }

    const fn threshold(self) -> usize {
        match self {
            Self::Sequential => usize::MAX,
            Self::Adaptive => 10_000,
        }
    }
}

fn metadata_value(entry_index: usize, field_index: usize) -> MetadataValue {
    MetadataValue {
        value: Some(metadata_value::Value::RawString(format!(
            "value-{}",
            (entry_index + field_index) % 100
        ))),
    }
}

fn entries(batch_size: usize, metadata_field_count: usize) -> Vec<(StoreKeyId, Arc<StoreValue>)> {
    (0..batch_size)
        .map(|entry_index| {
            let metadata = (0..metadata_field_count)
                .map(|field_index| {
                    (
                        format!("unindexed-{field_index}"),
                        metadata_value(entry_index, field_index),
                    )
                })
                .collect::<HashMap<_, _>>();

            (
                StoreKeyId(entry_index as u64),
                Arc::new(StoreValue { value: metadata }),
            )
        })
        .collect()
}

fn predicate_empty_index_ingestion(c: &mut Criterion) {
    let mut group = c.benchmark_group("predicate_empty_index_ingestion");
    group.sampling_mode(criterion::SamplingMode::Flat);

    for batch_size in BATCH_SIZES {
        for metadata_field_count in METADATA_FIELD_COUNTS {
            for policy in [
                MetadataFilteringPolicy::Sequential,
                MetadataFilteringPolicy::Adaptive,
            ] {
                let benchmark = EmptyPredicateIngestionBenchmark::new(
                    entries(batch_size, metadata_field_count),
                    policy.threshold(),
                );
                let scenario = format!(
                    "{}_entries_{}_metadata_fields_{}",
                    batch_size,
                    metadata_field_count,
                    policy.label()
                );

                benchmark.run_control();
                assert_eq!(benchmark.indexed_key_count(), 0);
                benchmark.run_candidate();
                assert_eq!(benchmark.indexed_key_count(), 0);

                group.throughput(Throughput::Elements(batch_size as u64));
                group.bench_with_input(
                    BenchmarkId::new("control", &scenario),
                    &benchmark,
                    |b, benchmark| {
                        b.iter(|| benchmark.run_control());
                    },
                );
                group.bench_with_input(
                    BenchmarkId::new("candidate", &scenario),
                    &benchmark,
                    |b, benchmark| {
                        b.iter(|| black_box(benchmark).run_candidate());
                    },
                );
            }
        }
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3));
    targets = predicate_empty_index_ingestion
}
criterion_main!(benches);
