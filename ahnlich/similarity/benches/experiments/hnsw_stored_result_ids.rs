use std::{num::NonZeroUsize, time::Duration};

use ahnlich_similarity::{
    EmbeddingKey, LinearAlgorithm,
    hnsw::{HNSWConfig, NodeId, get_node_id, index::HNSW},
};
use criterion::{BenchmarkId, Criterion, Throughput, black_box, criterion_group, criterion_main};

#[path = "../support/hnsw_collector_sharing.rs"]
mod support;

fn rehash_results(results: Vec<(EmbeddingKey, f32)>) -> Vec<(NodeId, f32)> {
    results
        .into_iter()
        .map(|(key, score)| (get_node_id(key.as_slice()), score))
        .collect()
}

fn bench(c: &mut Criterion) {
    let data = support::vectors(0, 10_000, 768);
    let queries = support::vectors(1_000_000, 32, 768);
    let index = HNSW::new_with_config_and_shared_collector_and_counter_shards(
        HNSWConfig {
            ef_construction: 200,
            maximum_connections: 16,
            maximum_connections_zero: 32,
            extend_candidates: false,
            keep_pruned_connections: false,
        },
        LinearAlgorithm::CosineSimilarity,
        NonZeroUsize::MIN,
    );
    index.insert(&data).unwrap();

    let k = NonZeroUsize::new(100).unwrap();
    let ef = Some(k);
    for query in &queries {
        let control = rehash_results(
            index
                .n_nearest_batched_cosine_with_ef(query.as_slice(), k, None, ef)
                .unwrap(),
        );
        let candidate = index
            .n_nearest_ids_with_ef(query.as_slice(), k, None, ef)
            .unwrap();
        assert_eq!(control.len(), k.get());
        assert_eq!(control.len(), candidate.len());
        for ((a, a_score), (b, b_score)) in control.iter().zip(&candidate) {
            assert_eq!(a, b);
            assert_eq!(a_score.to_bits(), b_score.to_bits());
        }
    }

    let mut group = c.benchmark_group("hnsw_stored_result_ids");
    let scenario = "10k_768d_k100_ef100";
    group.throughput(Throughput::Elements(queries.len() as u64));
    group.bench_function(BenchmarkId::new("control", scenario), |b| {
        b.iter(|| {
            for query in &queries {
                black_box(rehash_results(
                    index
                        .n_nearest_batched_cosine_with_ef(black_box(query.as_slice()), k, None, ef)
                        .unwrap(),
                ));
            }
        });
    });
    group.bench_function(BenchmarkId::new("candidate", scenario), |b| {
        b.iter(|| {
            for query in &queries {
                black_box(
                    index
                        .n_nearest_ids_with_ef(black_box(query.as_slice()), k, None, ef)
                        .unwrap(),
                );
            }
        });
    });
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .sample_size(30)
        .warm_up_time(Duration::from_secs(2))
        .measurement_time(Duration::from_secs(8));
    targets = bench
}
criterion_main!(benches);
