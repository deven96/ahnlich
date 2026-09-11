use std::num::NonZeroUsize;
use std::time::Duration;

use ahnlich_similarity::hnsw::index::HNSW;
use ahnlich_similarity::hnsw::{HNSWConfig, Node};
use ahnlich_similarity::{EmbeddingKey, LinearAlgorithm};
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};

const INDEX_SIZE: usize = 5_000;
const QUERY_COUNT: usize = 16;
const DIMENSIONS: [usize; 3] = [128, 768, 1_536];
const SEARCH_WIDTHS: [usize; 3] = [50, 100, 200];
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

fn vector(id: usize, dimensions: usize) -> Vec<f32> {
    (0..dimensions)
        .map(|dimension| {
            let mixed = id
                .wrapping_mul(1_664_525)
                .wrapping_add(dimension.wrapping_mul(1_013_904_223));
            ((mixed % 20_003) as f32 / 10_001.5) - 1.0
        })
        .collect()
}

fn build_index(dimensions: usize, metric: LinearAlgorithm) -> HNSW<LinearAlgorithm> {
    let config = HNSWConfig {
        ef_construction: 100,
        maximum_connections: 40,
        maximum_connections_zero: 80,
        extend_candidates: false,
        keep_pruned_connections: false,
    };
    let index = HNSW::new_with_config(config, metric);
    for id in 0..INDEX_SIZE {
        index
            .insert_node(Node::new(EmbeddingKey::new(vector(id, dimensions))))
            .unwrap();
    }
    index
}

fn queries(dimensions: usize) -> Vec<Vec<f32>> {
    (INDEX_SIZE..INDEX_SIZE + QUERY_COUNT)
        .map(|id| vector(id, dimensions))
        .collect()
}

fn hnsw_distance_reuse(c: &mut Criterion) {
    for dimensions in DIMENSIONS {
        for metric in METRICS {
            let index = build_index(dimensions, metric);
            let queries = queries(dimensions);
            let mut group = c.benchmark_group(format!(
                "hnsw_distance_reuse/{}d/{}",
                dimensions,
                metric_name(metric)
            ));
            group.throughput(Throughput::Elements(QUERY_COUNT as u64));
            let top_n = NonZeroUsize::new(10).unwrap();

            for ef in SEARCH_WIDTHS {
                for query in &queries {
                    let control = index.n_nearest_control(query, top_n, Some(ef)).unwrap();
                    let candidate = index
                        .n_nearest_reuse_distances(query, top_n, Some(ef))
                        .unwrap();
                    assert_eq!(
                        candidate, control,
                        "candidate: metric={metric:?}, dimensions={dimensions}, ef={ef}"
                    );
                }

                for variant in ["control", "candidate"] {
                    group.bench_with_input(BenchmarkId::new(variant, ef), &ef, |b, &ef| {
                        b.iter(|| {
                            for query in &queries {
                                let result = match variant {
                                    "control" => {
                                        index.n_nearest_control(black_box(query), top_n, Some(ef))
                                    }
                                    "candidate" => index.n_nearest_reuse_distances(
                                        black_box(query),
                                        top_n,
                                        Some(ef),
                                    ),
                                    _ => unreachable!(),
                                };
                                black_box(result.unwrap());
                            }
                        });
                    });
                }
            }
            group.finish();
        }
    }
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(10)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(3));
    targets = hnsw_distance_reuse
}
criterion_main!(benches);
