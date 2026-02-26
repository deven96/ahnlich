use std::collections::HashMap;

use crate::{
    EmbeddingKey,
    hnsw::{
        Node, NodeId,
        index::{HNSW, brute_knn},
    },
    tests::fixtures::mock_data::{MOST_SIMILAR, SEACH_TEXT, word_to_vector},
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

fn prepare_test_embeddings(
    dataset: &HashMap<String, Vec<f32>>,
    query_text: &str,
) -> (Vec<EmbeddingKey>, EmbeddingKey) {
    let query_vec = dataset
        .get(query_text)
        .map(|q| EmbeddingKey::new(q.clone()))
        .unwrap()
        .clone();

    let embeddings = dataset
        .iter()
        .filter(|(key, _)| key.as_str() != query_text)
        .map(|(_, v)| EmbeddingKey::new(v.clone()))
        .collect();

    (embeddings, query_vec)
}

fn load_raw_word_vectors() -> HashMap<String, Vec<f32>> {
    let words = std::fs::read_to_string("src/tests/fixtures/mock_data.json").unwrap();
    serde_json::from_str(&words).unwrap()
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
    let raw = load_raw_word_vectors();
    let dataset = word_to_vector();
    let (embeddings, query_vec) = prepare_test_embeddings(&raw, SEACH_TEXT);

    let hnsw = HNSW::default();
    hnsw.insert(embeddings).expect("Failed to batch insert");

    let query_node = Node::new(query_vec);
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
    let raw = load_raw_word_vectors();
    let dataset = word_to_vector();
    let (nodes, query_node) = prepare_test_data(&dataset, None);
    let (embeddings, _) = prepare_test_embeddings(&raw, SEACH_TEXT);

    let brute_search_results = brute_knn(&query_node, &nodes, MOST_SIMILAR.len())
        .into_iter()
        .map(|(node_id, _)| node_id)
        .collect::<Vec<_>>();

    let hnsw = HNSW::default();
    hnsw.insert(embeddings).expect("Failed to batch insert");

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
fn test_hnsw_average_recall_controlled() {
    let raw = load_raw_word_vectors();
    let dataset = word_to_vector();
    let k = MOST_SIMILAR.len();

    for (text, node_to_filter) in dataset.iter() {
        let (nodes, query_node) = prepare_test_data(&dataset, Some(node_to_filter.clone()));
        let (embeddings, _) = prepare_test_embeddings(&raw, text);

        // Rebuild HNSW for this iteration
        let hnsw = HNSW::default();
        hnsw.insert(embeddings).unwrap();

        let brute = brute_knn(&query_node, &nodes, k);

        // HNSW approximate neighbors
        let ann_ids: Vec<_> = hnsw
            .knn_search(&query_node, k, None)
            .expect("HNSW search failed");

        // Calculate recall
        let overlap: usize = brute.iter().filter(|(id, _)| ann_ids.contains(id)).count();
        let recall = overlap as f32 / k as f32;

        // Print detailed info
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

        assert!(
            recall >= 0.9,
            "Recall dropped to {:.3} — graph quality regressed",
            recall
        );
    }

    println!("✅ All queries passed recall test");
}

#[test]
fn test_recall_vs_ef_values() {
    println!("\n=== Recall vs ef values (dataset size=6, k=3) ===\n");

    let raw = load_raw_word_vectors();
    let dataset = word_to_vector();
    let k = MOST_SIMILAR.len();

    let ef_values = vec![3, 5, 10, 20, 50];

    for ef in ef_values {
        let mut total_recall = 0.0;
        let mut num_queries = 0;

        for (text, node_to_filter) in dataset.iter() {
            let (nodes, query_node) = prepare_test_data(&dataset, Some(node_to_filter.clone()));
            let (embeddings, _) = prepare_test_embeddings(&raw, text);

            let hnsw = HNSW::default();
            hnsw.insert(embeddings).unwrap();

            let brute = brute_knn(&query_node, &nodes, k);
            let ann_ids: Vec<_> = hnsw
                .knn_search(&query_node, k, Some(ef))
                .expect("HNSW search failed");

            let overlap: usize = brute.iter().filter(|(id, _)| ann_ids.contains(id)).count();
            let recall = overlap as f32 / k as f32;

            total_recall += recall;
            num_queries += 1;
        }

        let avg_recall = total_recall / num_queries as f32;
        println!(
            "ef = {:3} → Average Recall: {:.3} ({:5.1}%)",
            ef,
            avg_recall,
            avg_recall * 100.0
        );
    }

    println!("\nNote: With only 5 nodes in graph, even ef=3 explores most of the graph.");
}

fn load_synthetic_dataset(size: usize) -> HashMap<String, Node> {
    let filename = match size {
        100 => "src/tests/synthetic_embeddings_100.json",
        1000 => "src/tests/synthetic_embeddings_1k.json",
        _ => panic!("Unsupported dataset size"),
    };

    let data = std::fs::read_to_string(filename).expect(&format!("Failed to load {}", filename));
    let json: HashMap<String, Vec<f32>> =
        serde_json::from_str(&data).expect("Failed to parse JSON");

    json.into_iter()
        .map(|(key, value)| (key, Node::new(EmbeddingKey::new(value))))
        .collect()
}

#[test]
fn test_recall_vs_ef_on_realistic_dataset() {
    println!("\n=== Recall vs ef on 100-node dataset ===\n");

    let dataset = load_synthetic_dataset(100);
    let k = 10; // Search for 10 nearest neighbors

    println!("Dataset size: {}", dataset.len());
    println!("k (neighbors to find): {}", k);
    println!();

    let ef_values = vec![10, 20, 30, 50, 100];

    // Test on 10 random queries
    let test_queries: Vec<_> = dataset.values().take(10).cloned().collect();

    for ef in ef_values {
        let mut total_recall = 0.0;
        let mut num_queries = 0;

        for query_node in &test_queries {
            // Build graph with all nodes except query
            let nodes: Vec<Node> = dataset
                .values()
                .filter(|n| n.id() != query_node.id())
                .cloned()
                .collect();

            let embeddings: Vec<EmbeddingKey> = nodes
                .iter()
                .map(|n| EmbeddingKey::new(n.value().as_slice().to_vec()))
                .collect();

            let hnsw = HNSW::default();
            hnsw.insert(embeddings).unwrap();

            // Brute force ground truth
            let brute = brute_knn(&query_node, &nodes, k);

            // HNSW search
            let ann_ids: Vec<_> = hnsw
                .knn_search(&query_node, k, Some(ef))
                .expect("HNSW search failed");

            // Calculate recall
            let overlap: usize = brute.iter().filter(|(id, _)| ann_ids.contains(id)).count();
            let recall = overlap as f32 / k as f32;

            total_recall += recall;
            num_queries += 1;
        }

        let avg_recall = total_recall / num_queries as f32;
        println!(
            "ef = {:3} → Average Recall: {:.3} ({:5.1}%)",
            ef,
            avg_recall,
            avg_recall * 100.0
        );
    }
}

#[test]
fn test_recall_vs_ef_on_large_dataset() {
    println!("\n=== Recall vs ef on 1000-node dataset ===\n");

    let dataset = load_synthetic_dataset(1000);
    let k = 10;

    println!("Dataset size: {}", dataset.len());
    println!("k (neighbors to find): {}", k);
    println!();

    let ef_values = vec![10, 20, 50, 100, 200];

    // Test on 5 random queries (slower with 1000 nodes)
    let test_queries: Vec<_> = dataset.values().take(5).cloned().collect();

    for ef in ef_values {
        let mut total_recall = 0.0;
        let mut num_queries = 0;

        for query_node in &test_queries {
            let nodes: Vec<Node> = dataset
                .values()
                .filter(|n| n.id() != query_node.id())
                .cloned()
                .collect();

            let embeddings: Vec<EmbeddingKey> = nodes
                .iter()
                .map(|n| EmbeddingKey::new(n.value().as_slice().to_vec()))
                .collect();

            let hnsw = HNSW::default();
            hnsw.insert(embeddings).unwrap();

            let brute = brute_knn(&query_node, &nodes, k);
            let ann_ids: Vec<_> = hnsw
                .knn_search(&query_node, k, Some(ef))
                .expect("HNSW search failed");

            let overlap: usize = brute.iter().filter(|(id, _)| ann_ids.contains(id)).count();
            let recall = overlap as f32 / k as f32;

            total_recall += recall;
            num_queries += 1;
        }

        let avg_recall = total_recall / num_queries as f32;
        println!(
            "ef = {:3} → Average Recall: {:.3} ({:5.1}%)",
            ef,
            avg_recall,
            avg_recall * 100.0
        );
    }
}
