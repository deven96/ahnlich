use std::time::Duration;

use ahnlich_similarity::hnsw::HNSWConfig;
use ahnlich_similarity::hnsw::index::HNSW;
use ahnlich_similarity::{EmbeddingKey, LinearAlgorithm};
use criterion::{
    BatchSize, BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main,
};

const FULL_BUILD_COUNT: usize = 1_000;
const INCREMENTAL_BASE_COUNT: usize = 2_500;
const INCREMENTAL_INSERT_COUNT: usize = 100;
const DIMENSIONS: [usize; 3] = [128, 768, 1_536];
const METRICS: [LinearAlgorithm; 3] = [
    LinearAlgorithm::EuclideanDistance,
    LinearAlgorithm::CosineSimilarity,
    LinearAlgorithm::DotProductSimilarity,
];

fn metric_name(metric: LinearAlgorithm) -> &'static str {
    match metric {
        LinearAlgorithm::EuclideanDistance => "euclidean",
        LinearAlgorithm::CosineSimilarity => "cosine",
        LinearAlgorithm::DotProductSimilarity => "dotproduct",
    }
}

fn new_index(metric: LinearAlgorithm) -> HNSW<LinearAlgorithm> {
    HNSW::new_with_config(
        HNSWConfig {
            ef_construction: 100,
            maximum_connections: 40,
            maximum_connections_zero: 80,
            extend_candidates: false,
            keep_pruned_connections: false,
        },
        metric,
    )
}

fn vectors(start: usize, count: usize, dimensions: usize) -> Vec<EmbeddingKey> {
    (start..start + count)
        .map(|id| {
            EmbeddingKey::new(
                (0..dimensions)
                    .map(|dimension| {
                        let mixed = id
                            .wrapping_mul(1_664_525)
                            .wrapping_add(dimension.wrapping_mul(1_013_904_223));
                        ((mixed % 20_003) as f32 / 10_001.5) - 1.0
                    })
                    .collect(),
            )
        })
        .collect()
}

fn assert_equivalent_graphs(
    metric: LinearAlgorithm,
    dimensions: usize,
    base: &[EmbeddingKey],
    additions: &[EmbeddingKey],
) {
    let control = new_index(metric);
    control.insert_control_for_benchmark(base).unwrap();
    control.insert_control_for_benchmark(additions).unwrap();

    let candidate = new_index(metric);
    candidate.insert_reuse_distances_candidate(base).unwrap();
    candidate
        .insert_reuse_distances_candidate(additions)
        .unwrap();

    assert_eq!(
        candidate.topology_snapshot(),
        control.topology_snapshot(),
        "candidate graph differs: metric={metric:?}, dimensions={dimensions}"
    );
}

fn hnsw_insert_distance_reuse(c: &mut Criterion) {
    for dimensions in DIMENSIONS {
        let full_build = vectors(0, FULL_BUILD_COUNT, dimensions);
        let incremental_base = vectors(0, INCREMENTAL_BASE_COUNT, dimensions);
        let incremental_additions =
            vectors(INCREMENTAL_BASE_COUNT, INCREMENTAL_INSERT_COUNT, dimensions);

        for metric in METRICS {
            assert_equivalent_graphs(
                metric,
                dimensions,
                &incremental_base,
                &incremental_additions,
            );

            let mut full_group = c.benchmark_group(format!(
                "hnsw_insert_distance_reuse/full/{}d/{}",
                dimensions,
                metric_name(metric)
            ));
            full_group.throughput(Throughput::Elements(FULL_BUILD_COUNT as u64));
            for variant in ["control", "candidate"] {
                full_group.bench_with_input(
                    BenchmarkId::new(variant, FULL_BUILD_COUNT),
                    &variant,
                    |b, &variant| {
                        b.iter_batched(
                            || new_index(metric),
                            |index| {
                                if variant == "control" {
                                    index
                                        .insert_control_for_benchmark(black_box(&full_build))
                                        .unwrap();
                                } else {
                                    index
                                        .insert_reuse_distances_candidate(black_box(&full_build))
                                        .unwrap();
                                }
                                black_box(index);
                            },
                            BatchSize::LargeInput,
                        );
                    },
                );
            }
            full_group.finish();

            let mut incremental_group = c.benchmark_group(format!(
                "hnsw_insert_distance_reuse/incremental/{}d/{}",
                dimensions,
                metric_name(metric)
            ));
            incremental_group.throughput(Throughput::Elements(INCREMENTAL_INSERT_COUNT as u64));
            let incremental_seed = new_index(metric);
            incremental_seed
                .insert_control_for_benchmark(&incremental_base)
                .unwrap();
            for variant in ["control", "candidate"] {
                incremental_group.bench_with_input(
                    BenchmarkId::new(variant, INCREMENTAL_INSERT_COUNT),
                    &variant,
                    |b, &variant| {
                        b.iter_batched(
                            || incremental_seed.clone_for_benchmark(),
                            |index| {
                                if variant == "control" {
                                    index
                                        .insert_control_for_benchmark(black_box(
                                            &incremental_additions,
                                        ))
                                        .unwrap();
                                } else {
                                    index
                                        .insert_reuse_distances_candidate(black_box(
                                            &incremental_additions,
                                        ))
                                        .unwrap();
                                }
                                black_box(index);
                            },
                            BatchSize::LargeInput,
                        );
                    },
                );
            }
            incremental_group.finish();
        }
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3));
    targets = hnsw_insert_distance_reuse
}
criterion_main!(benches);
