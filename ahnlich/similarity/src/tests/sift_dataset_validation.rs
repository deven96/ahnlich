use crate::{
    DistanceFn, LinearAlgorithm,
    hnsw::{HNSWConfig, get_node_id},
    tests::fixtures::sift::{AnnDataset, load_dataset},
};
use rstest::rstest;

use crate::hnsw::{Node, index::HNSW};

const K: usize = 50; // number of neighbors to check recall for

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
    let mut hnsw = HNSW::new_with_config(config, distance_algorithm);
    for vec in vectors.iter() {
        let node = Node::new(vec.clone());
        hnsw.insert(node).unwrap();
    }
    hnsw
}

fn compute_recall_for_config(dataset: &AnnDataset, config: HNSWConfig, k: usize) -> f32 {
    let hnsw = build_hnsw_from_vectors(
        &dataset.sift_data,
        config,
        LinearAlgorithm::EuclideanDistance,
    );

    let mut total_recall = 0.0;

    for (query_idx, query_vec) in dataset.sift_query.iter().enumerate() {
        let query_node = Node::new(query_vec.clone());

        let ann_ids = hnsw.knn_search(&query_node, K, Some(16)).unwrap();

        let true_neighbors = &dataset.ground_truth[query_idx];

        let overlap = true_neighbors
            .iter()
            .take(k)
            .filter(|&&idx| {
                let neighbor_vec = &dataset.sift_data[idx as usize];
                let neighbor_id = get_node_id(neighbor_vec);
                ann_ids.contains(&neighbor_id)
            })
            .count();

        total_recall += overlap as f32 / K as f32;
    }

    total_recall / dataset.sift_query.len() as f32
}

#[test]
fn test_hnsw_recall_sift10k() {
    let dataset = load_dataset();

    let config = HNSWConfig {
        ef_construction: 100,
        maximum_connections: 40,
        maximum_connections_zero: 80,

        keep_pruned_connections: false,
        extend_candidates: false,
    };

    let avg_recall = compute_recall_for_config(&dataset, config, K);

    println!("Average recall: {:.4}", avg_recall);

    assert!(avg_recall > 0.90);
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

    let recall = compute_recall_for_config(&dataset, config.hnsw.clone(), K);

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
