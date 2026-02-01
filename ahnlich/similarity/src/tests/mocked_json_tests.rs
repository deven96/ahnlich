use std::collections::HashMap;

use crate::{
    hnsw::{
        Node, NodeId,
        index::{HNSW, brute_knn},
    },
    tests::{MOST_SIMILAR, SEACH_TEXT, word_to_vector},
};

fn prepare_test_data(
    dataset: &HashMap<String, Node>,
    query_node: Option<Node>,
) -> (Vec<Node>, Node) {
    let filter = query_node.unwrap_or(dataset.get(SEACH_TEXT).unwrap().clone());

    let nodes = dataset
        .clone()
        .into_iter()
        .filter_map(|(_, node)| {
            if node.id() == filter.id() {
                None
            } else {
                Some(node)
            }
        })
        .collect();

    (nodes, filter)
}

pub fn average_recall(dataset: &HashMap<String, Node>, hnsw: &HNSW, k: usize) -> f32 {
    let mut total_recall = 0.0;
    let mut queries = 0;

    for node_to_filter in dataset.values() {
        // All other nodes except the query

        let (nodes, query_node) = prepare_test_data(&dataset, Some(node_to_filter.clone()));

        // Brute-force KNN ground truth
        let brute = brute_knn(&query_node, &nodes, k);

        // HNSW approximate neighbors
        let ann_ids: Vec<_> = hnsw
            .knn_search(&query_node, k, None)
            .expect("HNSW search failed");

        // Calculate overlap
        let overlap: usize = brute.iter().filter(|(id, _)| ann_ids.contains(id)).count();

        let recall = overlap as f32 / k as f32;

        // **Print detailed query info**
        let missing: Vec<&NodeId> = brute
            .iter()
            .filter_map(|(id, _)| {
                if !ann_ids.contains(id) {
                    Some(id)
                } else {
                    None
                }
            })
            .collect();

        println!("Query: {:?}", query_node.id());
        println!("  Recall: {:.3}", recall);
        println!("  Returned neighbors: {:?}", ann_ids);
        println!("  Missing neighbors: {:?}", missing);
        println!(
            "  True nearest neighbors: {:?}",
            brute.iter().map(|(id, _)| id).collect::<Vec<_>>()
        );

        total_recall += recall;
        queries += 1;
    }

    let avg = total_recall / queries as f32;
    println!("Average recall@{}: {:.3}", k, avg);

    avg
}

#[test]
fn test_simple_brute_knn_works() {
    let dataset = word_to_vector();

    let (nodes, query_node) = prepare_test_data(&dataset, None);

    let search_results = brute_knn(&query_node, &nodes, 1);

    assert_eq!(search_results.len(), 1);

    // get first most similar n
    let most_similar = dataset.get(MOST_SIMILAR[0]).unwrap();

    let first = search_results.first().unwrap();
    assert_eq!(&first.0, most_similar.id());

    let search_results = brute_knn(&query_node, &nodes, MOST_SIMILAR.len())
        .into_iter()
        .map(|(node_id, _)| node_id)
        .collect::<Vec<_>>();

    for similar in MOST_SIMILAR {
        let most_similar = dataset.get(similar).unwrap();
        assert!(search_results.contains(most_similar.id()))
    }
}

#[test]
fn test_hnsw_simple_setup_works() {
    let dataset = word_to_vector();
    let (nodes, query_node) = prepare_test_data(&dataset, None);

    let mut hnsw = HNSW::default();

    for node in nodes.into_iter() {
        hnsw.insert(node).expect("Failed to insert hnsw node");
    }

    let search_results = hnsw
        .knn_search(&query_node, 1, None)
        .expect("Failed to search hnsw");

    assert_eq!(search_results.len(), 1);

    // get first most similar n
    let most_similar = dataset.get(MOST_SIMILAR[0]).unwrap();

    let first = search_results.first().unwrap();

    assert_eq!(first, most_similar.id());

    let search_results = hnsw
        .knn_search(&query_node, MOST_SIMILAR.len(), None)
        .expect("Failed to search hnsw");

    assert_eq!(search_results.len(), MOST_SIMILAR.len());

    for similar in MOST_SIMILAR {
        let most_similar = dataset.get(similar).unwrap();
        assert!(search_results.contains(most_similar.id()))
    }
}

#[test]
fn test_hnsw_recall_on_simple_setup() {
    let dataset = word_to_vector();
    let (nodes, query_node) = prepare_test_data(&dataset, None);

    let brute_search_results = brute_knn(&query_node, &nodes, MOST_SIMILAR.len())
        .into_iter()
        .map(|(node_id, _)| node_id)
        .collect::<Vec<_>>();

    let mut hnsw = HNSW::default();

    for node in nodes.into_iter() {
        hnsw.insert(node).expect("Failed to insert hnsw node");
    }

    let hnsw_search_results = hnsw
        .knn_search(&query_node, MOST_SIMILAR.len(), None)
        .expect("Failed to search hnsw");

    let overlap = brute_search_results
        .iter()
        .filter(|id| hnsw_search_results.contains(id))
        .count();

    let recall = overlap as f32 / MOST_SIMILAR.len() as f32;

    assert!(recall > 0.9);
}

#[test]
fn test_hnsw_average_recall() {
    let dataset = word_to_vector();

    let mut hnsw = HNSW::default();

    for node in dataset.values().cloned() {
        hnsw.insert(node).unwrap();
    }

    let recall = average_recall(&dataset, &hnsw, MOST_SIMILAR.len());

    assert!(
        recall >= 0.9,
        "Recall dropped to {:.3} — graph quality regressed",
        recall
    );

    println!(
        "✅ Test passed: average recall@{} = {:.3}",
        MOST_SIMILAR.len(),
        recall
    );
}
