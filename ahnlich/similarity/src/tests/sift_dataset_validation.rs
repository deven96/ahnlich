use crate::{
    DistanceFn, EmbeddingKey, LinearAlgorithm,
    hnsw::{HNSWConfig, NodeId, get_node_id},
    tests::fixtures::sift::{AnnDataset, load_dataset},
};
use rstest::rstest;

use crate::hnsw::{Node, index::HNSW};

const K: usize = 50; // number of neighbors to check recall for

/// Every metric the index accepts. Recall is measured for all of them: an index that
/// ranks one metric backwards scores 0.0 here, which is how the inverted ordering of
/// cosine and dot product went unnoticed while Euclidean-only tests passed.
const METRICS: [LinearAlgorithm; 3] = [
    LinearAlgorithm::EuclideanDistance,
    LinearAlgorithm::CosineSimilarity,
    LinearAlgorithm::DotProductSimilarity,
];

#[derive(Clone)]
struct ExperimentConfig {
    hnsw: HNSWConfig,
    knn_search_ef_param: usize,
    recall_threshold: f32,
}

fn build_hnsw_from_vectors<D: DistanceFn>(
    vectors: &[Vec<f32>],
    config: HNSWConfig,
    distance_algorithm: D,
) -> HNSW<D> {
    let hnsw = HNSW::new_with_config(config, distance_algorithm);
    let embeddings: Vec<EmbeddingKey> = vectors
        .iter()
        .map(|v| EmbeddingKey::new(v.clone()))
        .collect();
    hnsw.insert(&embeddings).unwrap();
    hnsw
}

/// Exact top-k for one query, by brute force under `metric`.
///
/// Ground truth is computed rather than read from `siftsmall_groundtruth.ivecs`: that
/// file is Euclidean, so it is only a valid target for one of the three metrics. The
/// vectors themselves are metric-agnostic.
fn brute_force_top_k(
    base: &[Vec<f32>],
    query: &[f32],
    metric: LinearAlgorithm,
    k: usize,
) -> Vec<NodeId> {
    let mut scored: Vec<(&Vec<f32>, _)> = base
        .iter()
        .map(|vector| (vector, metric.closeness(query, vector)))
        .collect();

    scored.sort_by(|(_, a), (_, b)| b.cmp(a));
    scored
        .into_iter()
        .take(k)
        .map(|(vector, _)| get_node_id(vector))
        .collect()
}

fn compute_recall_for_config(
    dataset: &AnnDataset,
    config: HNSWConfig,
    k: usize,
    metric: LinearAlgorithm,
) -> f32 {
    let hnsw = build_hnsw_from_vectors(&dataset.sift_data, config, metric);

    let mut total_recall = 0.0;

    for query_vec in dataset.sift_query.iter() {
        let query_node = Node::new(EmbeddingKey::new(query_vec.clone()));

        let ann_ids = hnsw.knn_search(&query_node, K, Some(16), None).unwrap();
        let true_neighbors = brute_force_top_k(&dataset.sift_data, query_vec, metric, k);

        let overlap = true_neighbors
            .iter()
            .filter(|id| ann_ids.contains(id))
            .count();

        total_recall += overlap as f32 / K as f32;
    }

    total_recall / dataset.sift_query.len() as f32
}

#[rstest]
#[case(LinearAlgorithm::EuclideanDistance)]
#[case(LinearAlgorithm::CosineSimilarity)]
#[case(LinearAlgorithm::DotProductSimilarity)]
fn test_hnsw_recall_sift10k(#[case] metric: LinearAlgorithm) {
    let dataset = load_dataset();

    let config = HNSWConfig {
        ef_construction: 100,
        maximum_connections: 40,
        maximum_connections_zero: 80,

        keep_pruned_connections: false,
        extend_candidates: false,
    };

    let avg_recall = compute_recall_for_config(&dataset, config, K, metric);

    println!("{metric:?}: average recall = {avg_recall:.4}");

    assert!(avg_recall > 0.90, "{metric:?} recall {avg_recall:.4}");
}

/// A vector is closer to itself than to its opposite. True of every metric, whatever
/// it computes, so this catches a distance function whose return type claims the wrong
/// direction — the one thing the type system cannot check.
#[test]
fn closeness_points_the_right_way_for_every_metric() {
    let vector: Vec<f32> = (0..8).map(|i| (i as f32 + 1.0) * 0.25).collect();
    let opposite: Vec<f32> = vector.iter().map(|x| -x).collect();

    for metric in METRICS {
        assert!(
            metric.closeness(&vector, &vector) > metric.closeness(&vector, &opposite),
            "{metric:?}: a vector is not ranked closer to itself than to its opposite"
        );
    }
}

#[rstest]
#[case(ExperimentConfig {
    hnsw: HNSWConfig {
        ef_construction: 50,
        maximum_connections: 16,
        maximum_connections_zero: 32,

        keep_pruned_connections: false,
        extend_candidates: false

    },
    knn_search_ef_param: 16,
    recall_threshold: 0.80,
})]
#[case(ExperimentConfig {
    hnsw: HNSWConfig {
        ef_construction: 20,
        maximum_connections: 5,
        maximum_connections_zero: 10,

        keep_pruned_connections: false,
        extend_candidates: false
    },
    knn_search_ef_param: 5,
    recall_threshold: 0.80,
})]
#[case(ExperimentConfig {
    hnsw: HNSWConfig {
        ef_construction: 50,
        maximum_connections: 25,
        maximum_connections_zero: 50,
        keep_pruned_connections: false,
        extend_candidates: false

    },
    knn_search_ef_param: 20,
    recall_threshold: 0.90,
})]
fn recall_experiment(#[case] config: ExperimentConfig) {
    let dataset = load_dataset();

    let recall =
        compute_recall_for_config(&dataset, config.hnsw, K, LinearAlgorithm::EuclideanDistance);

    println!(
        "M={}, ef_construction={}, ef_search={}, recall={:.4}",
        config.hnsw.maximum_connections,
        config.hnsw.ef_construction,
        config.knn_search_ef_param,
        recall
    );

    assert!(
        recall >= config.recall_threshold,
        "Recall {:.4} below threshold {:.4}",
        recall,
        config.recall_threshold
    );
}
