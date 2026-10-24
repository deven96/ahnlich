//! End-to-end HNSW candidate: compact per-layer outgoing adjacency lists.
use std::{num::NonZeroUsize, time::Duration};

use ahnlich_similarity::{
    LinearAlgorithm,
    hnsw::{
        CompactEdges, HNSWConfig,
        index::{HNSW, experiments::ControlHNSW},
    },
};
use criterion::{
    BatchSize, BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main,
};

#[path = "../support/hnsw_collector_sharing.rs"]
mod support;

const CONFIG: HNSWConfig = HNSWConfig {
    ef_construction: 200,
    maximum_connections: 16,
    maximum_connections_zero: 32,
    extend_candidates: false,
    keep_pruned_connections: false,
};
const DIMENSION: usize = 768;
const BUILD_COUNT: usize = 1_000;
const BASE_COUNT: usize = 2_500;
const ADDITION_COUNT: usize = 100;
const SEARCH_COUNT: usize = 10_000;

fn control() -> ControlHNSW<LinearAlgorithm> {
    ControlHNSW::new_control_with_config_and_shared_collector_and_counter_shards(
        CONFIG,
        LinearAlgorithm::CosineSimilarity,
        NonZeroUsize::new(1).unwrap(),
    )
}

fn candidate() -> HNSW<LinearAlgorithm, CompactEdges> {
    HNSW::new_with_config_and_shared_collector_and_counter_shards(
        CONFIG,
        LinearAlgorithm::CosineSimilarity,
        NonZeroUsize::new(1).unwrap(),
    )
}

fn assert_equivalent(
    data: &[ahnlich_similarity::EmbeddingKey],
    queries: &[ahnlich_similarity::EmbeddingKey],
) {
    let control = control();
    control.insert(data).unwrap();
    let candidate = candidate();
    candidate.insert(data).unwrap();
    assert_eq!(candidate.topology_snapshot(), control.topology_snapshot());
    for query in queries {
        assert_eq!(
            candidate
                .n_nearest_with_ef(
                    query.as_slice(),
                    NonZeroUsize::new(10).unwrap(),
                    None,
                    NonZeroUsize::new(100)
                )
                .unwrap(),
            control
                .n_nearest_with_ef(
                    query.as_slice(),
                    NonZeroUsize::new(10).unwrap(),
                    None,
                    NonZeroUsize::new(100)
                )
                .unwrap(),
        );
    }
}

fn benchmark(c: &mut Criterion) {
    let full = support::vectors(0, BUILD_COUNT, DIMENSION);
    let base = support::vectors(0, BASE_COUNT, DIMENSION);
    let additions = support::vectors(BASE_COUNT, ADDITION_COUNT, DIMENSION);
    let search_data = support::vectors(0, SEARCH_COUNT, DIMENSION);
    let queries = support::vectors(1_000_000, 16, DIMENSION);

    assert_equivalent(&full, &queries[..4]);

    let mut build = c.benchmark_group("hnsw_compact_adjacency_graph/build");
    build.sampling_mode(criterion::SamplingMode::Flat);
    build.throughput(Throughput::Elements(BUILD_COUNT as u64));
    for variant in ["control", "candidate"] {
        build.bench_function(BenchmarkId::new(variant, "1k_768d"), |b| {
            b.iter_batched(
                || (),
                |_| {
                    if variant == "control" {
                        let index = control();
                        index.insert(black_box(&full)).unwrap();
                        black_box(index);
                    } else {
                        let index = candidate();
                        index.insert(black_box(&full)).unwrap();
                        black_box(index);
                    }
                },
                BatchSize::LargeInput,
            );
        });
    }
    build.finish();

    let control_seed = control();
    control_seed.insert(&base).unwrap();
    let candidate_seed = candidate();
    candidate_seed.insert(&base).unwrap();
    assert_eq!(
        candidate_seed.topology_snapshot(),
        control_seed.topology_snapshot()
    );

    let mut existing = c.benchmark_group("hnsw_compact_adjacency_graph/existing_insert");
    existing.sampling_mode(criterion::SamplingMode::Flat);
    existing.throughput(Throughput::Elements(ADDITION_COUNT as u64));
    for variant in ["control", "candidate"] {
        existing.bench_function(BenchmarkId::new(variant, "2_5k_plus_100_768d"), |b| {
            b.iter_batched(
                || {
                    if variant == "control" {
                        let index = control();
                        index.insert(&base).unwrap();
                        Either::Control(index)
                    } else {
                        let index = candidate();
                        index.insert(&base).unwrap();
                        Either::Candidate(index)
                    }
                },
                |index| {
                    match &index {
                        Either::Control(index) => index.insert(black_box(&additions)).unwrap(),
                        Either::Candidate(index) => index.insert(black_box(&additions)).unwrap(),
                    }
                    black_box(index);
                },
                BatchSize::LargeInput,
            );
        });
    }
    existing.finish();

    assert_equivalent(&search_data, &queries[..4]);
    let control_search = control();
    control_search.insert(&search_data).unwrap();
    let candidate_search = candidate();
    candidate_search.insert(&search_data).unwrap();
    let mut search = c.benchmark_group("hnsw_compact_adjacency_graph/search");
    search.throughput(Throughput::Elements(queries.len() as u64));
    for variant in ["control", "candidate"] {
        search.bench_function(BenchmarkId::new(variant, "10k_768d_ef100"), |b| {
            b.iter(|| {
                for query in &queries {
                    if variant == "control" {
                        black_box(
                            control_search
                                .n_nearest_with_ef(
                                    black_box(query.as_slice()),
                                    NonZeroUsize::new(10).unwrap(),
                                    None,
                                    NonZeroUsize::new(100),
                                )
                                .unwrap(),
                        );
                    } else {
                        black_box(
                            candidate_search
                                .n_nearest_with_ef(
                                    black_box(query.as_slice()),
                                    NonZeroUsize::new(10).unwrap(),
                                    None,
                                    NonZeroUsize::new(100),
                                )
                                .unwrap(),
                        );
                    }
                }
            });
        });
    }
    search.finish();
}

enum Either {
    Control(ControlHNSW<LinearAlgorithm>),
    Candidate(HNSW<LinearAlgorithm, CompactEdges>),
}

criterion_group! {
    name = benches;
    config = Criterion::default().sample_size(10)
        .warm_up_time(Duration::from_secs(1)).measurement_time(Duration::from_secs(3));
    targets = benchmark
}
criterion_main!(benches);
