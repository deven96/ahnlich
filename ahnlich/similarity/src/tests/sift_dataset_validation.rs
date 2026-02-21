use crate::hnsw::get_node_id;
use crate::tests::datasets::AnnDataset;
use rstest::rstest;

use crate::hnsw::{Node, index::HNSW};
use crate::tests::datasets::loader::load_dataset;

const K: usize = 50; // number of neighbors to check recall for

#[derive(Clone, Copy)]
struct HNSWConfig {
    ef_construction: usize,
    maximum_connections: usize,
    maximum_connections_zero: Option<usize>,
    knn_search_ef_param: usize,
}

#[derive(Clone)]
struct ExperimentConfig {
    hnsw: HNSWConfig,
    recall_threshold: f32,
}

fn build_hnsw_from_vectors(vectors: &[Vec<f32>], config: HNSWConfig) -> HNSW {
    let mut hnsw = HNSW::new(
        config.ef_construction,
        config.maximum_connections,
        config.maximum_connections_zero,
    );
    for vec in vectors.iter() {
        let node = Node::new(vec.clone());
        hnsw.insert(node).unwrap();
    }
    hnsw
}

fn compute_recall_for_config(dataset: &AnnDataset, config: HNSWConfig, k: usize) -> f32 {
    let hnsw = build_hnsw_from_vectors(&dataset.sift_data, config);

    let mut total_recall = 0.0;

    for (query_idx, query_vec) in dataset.sift_query.iter().enumerate() {
        let query_node = Node::new(query_vec.clone());

        let ann_ids = hnsw
            .knn_search(&query_node, K, Some(config.knn_search_ef_param))
            .unwrap();

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
        maximum_connections_zero: Some(80),

        knn_search_ef_param: 16,
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
        maximum_connections_zero: Some(32),
        knn_search_ef_param: 16,
    },
    recall_threshold: 0.80,
})]
#[case(ExperimentConfig {
    hnsw: HNSWConfig {
        ef_construction: 20,
        maximum_connections: 5,
        maximum_connections_zero: Some(10),
        knn_search_ef_param: 5,
    },
    recall_threshold: 0.80,
})]
#[case(ExperimentConfig {
    hnsw: HNSWConfig {
        ef_construction: 50,
        maximum_connections: 25,
        maximum_connections_zero: Some(50),
        knn_search_ef_param: 20,
    },
    recall_threshold: 0.90,
})]
fn recall_experiment(#[case] config: ExperimentConfig) {
    let dataset = load_dataset();

    let recall = compute_recall_for_config(&dataset, config.hnsw.clone(), K);

    println!(
        "M={}, ef_construction={}, ef_search={}, recall={:.4}",
        config.hnsw.maximum_connections,
        config.hnsw.ef_construction,
        config.hnsw.knn_search_ef_param,
        recall
    );

    assert!(
        recall >= config.recall_threshold,
        "Recall {:.4} below threshold {:.4}",
        recall,
        config.recall_threshold
    );
}
