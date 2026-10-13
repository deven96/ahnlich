#[path = "../support/hnsw_collector_sharing.rs"]
mod support;

use criterion::{
    BatchSize, BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main,
};
use std::{num::NonZeroUsize, time::Duration};
use support::{PERFORMANCE_SCENARIOS, Variant, index, populated, vectors};

fn hnsw_collector_sharing(c: &mut Criterion) {
    let mut construction = c.benchmark_group("hnsw_collector_sharing/construction_and_drop");
    for variant in Variant::ALL {
        construction.bench_function(BenchmarkId::new(variant.label(), "empty"), |b| {
            b.iter(|| black_box(index(variant, PERFORMANCE_SCENARIOS[0])));
        });
    }
    construction.finish();

    for scenario in PERFORMANCE_SCENARIOS {
        let data = vectors(0, scenario.entries, scenario.dimension);
        let incoming = vectors(scenario.entries, 100, scenario.dimension);
        let queries = vectors(1_000_000, 16, scenario.dimension);
        let n = NonZeroUsize::new(10).unwrap();
        let ef = NonZeroUsize::new(scenario.ef_search);
        // Validate paired outputs before timing. Each index sees identical data/order.
        let fixtures = Variant::ALL.map(|variant| populated(variant, scenario, &data));
        for query in &queries {
            assert_eq!(
                fixtures[0]
                    .n_nearest_with_ef(query.as_slice(), n, None, ef)
                    .unwrap(),
                fixtures[1]
                    .n_nearest_with_ef(query.as_slice(), n, None, ef)
                    .unwrap()
            );
        }
        drop(fixtures);

        let mut ingestion = c.benchmark_group("hnsw_collector_sharing/initial_insert");
        ingestion.sampling_mode(criterion::SamplingMode::Flat);
        ingestion.throughput(Throughput::Elements(scenario.entries as u64));
        for variant in Variant::ALL {
            ingestion.bench_function(BenchmarkId::new(variant.label(), scenario.label), |b| {
                b.iter_batched(
                    || index(variant, scenario),
                    |index| {
                        index.insert(black_box(&data)).unwrap();
                        black_box(index) // Returned index is dropped outside the timed routine.
                    },
                    BatchSize::PerIteration,
                );
            });
        }
        ingestion.finish();

        let mut incremental = c.benchmark_group("hnsw_collector_sharing/existing_insert");
        incremental.sampling_mode(criterion::SamplingMode::Flat);
        incremental.throughput(Throughput::Elements(incoming.len() as u64));
        for variant in Variant::ALL {
            incremental.bench_function(BenchmarkId::new(variant.label(), scenario.label), |b| {
                b.iter_batched(
                    || populated(variant, scenario, &data),
                    |index| {
                        index.insert(black_box(&incoming)).unwrap();
                        black_box(index)
                    },
                    BatchSize::PerIteration,
                );
            });
        }
        incremental.finish();

        let mut search = c.benchmark_group("hnsw_collector_sharing/search");
        search.throughput(Throughput::Elements(queries.len() as u64));
        for variant in Variant::ALL {
            let fixture = populated(variant, scenario, &data);
            search.bench_function(BenchmarkId::new(variant.label(), scenario.label), |b| {
                b.iter(|| {
                    for query in &queries {
                        black_box(
                            fixture
                                .n_nearest_with_ef(black_box(query.as_slice()), n, None, ef)
                                .unwrap(),
                        );
                    }
                });
            });
        }
        search.finish();

        let mut deletion = c.benchmark_group("hnsw_collector_sharing/delete");
        deletion.throughput(Throughput::Elements(data.len() as u64));
        for variant in Variant::ALL {
            deletion.bench_function(BenchmarkId::new(variant.label(), scenario.label), |b| {
                b.iter_batched(
                    || populated(variant, scenario, &data),
                    |index| {
                        assert_eq!(index.delete(black_box(&data)).unwrap(), data.len());
                        black_box(index)
                    },
                    BatchSize::PerIteration,
                );
            });
        }
        deletion.finish();

        let mut destruction = c.benchmark_group("hnsw_collector_sharing/drop_populated");
        for variant in Variant::ALL {
            destruction.bench_function(BenchmarkId::new(variant.label(), scenario.label), |b| {
                b.iter_batched(
                    || populated(variant, scenario, &data),
                    |index| drop(black_box(index)),
                    BatchSize::PerIteration,
                );
            });
        }
        destruction.finish();
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10)
        .warm_up_time(Duration::from_secs(1)).measurement_time(Duration::from_secs(3));
    targets = hnsw_collector_sharing
}
criterion_main!(benches);
