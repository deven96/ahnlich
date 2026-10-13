use super::*;
use papaya::Guard;
use rand::{Rng, SeedableRng, rngs::StdRng};

fn fixture() -> Vec<EmbeddingKey> {
    let mut rng = StdRng::seed_from_u64(424);
    (0..128)
        .map(|_| EmbeddingKey::new((0..8).map(|_| rng.gen_range(-2.0..2.0)).collect()))
        .collect()
}

fn candidate(metric: LinearAlgorithm) -> HNSW<LinearAlgorithm> {
    HNSW::new_candidate(metric, 100, 4, 8, false, false)
}

fn assert_shared(index: &HNSW<LinearAlgorithm>) {
    let collector = index.collector.as_ref().unwrap();
    assert!(std::ptr::eq(
        index.nodes.guard().collector(),
        collector.as_ref()
    ));
    assert!(std::ptr::eq(
        index.graph.guard().collector(),
        collector.as_ref()
    ));
    for set in index.graph.pin().values() {
        assert!(std::ptr::eq(set.guard().collector(), collector.as_ref()));
    }
    for node in index.nodes.pin().values() {
        assert!(std::ptr::eq(
            node.neighbours.guard().collector(),
            collector.as_ref()
        ));
        assert!(std::ptr::eq(
            node.back_links.guard().collector(),
            collector.as_ref()
        ));
        for (layer, set) in node.neighbours.pin().iter() {
            assert!(std::ptr::eq(set.guard().collector(), collector.as_ref()));
            let maximum = if layer.0 == 0 {
                index.maximum_connections_zero
            } else {
                index.maximum_connections
            };
            assert!(set.pin().len() <= maximum);
        }
    }
}

#[test]
fn collector_sharing_survives_insertion_pruning_and_deletion() {
    let data = fixture();
    let index = candidate(LinearAlgorithm::EuclideanDistance);
    let separate = candidate(LinearAlgorithm::EuclideanDistance);
    assert!(!Arc::ptr_eq(
        index.collector.as_ref().unwrap(),
        separate.collector.as_ref().unwrap()
    ));
    index.insert_node(Node::new(data[0].clone())).unwrap();
    assert_shared(&index);
    index.insert(&data).unwrap();
    assert_eq!(index.nodes.pin().len(), data.len());
    assert_shared(&index);
    // Avoid deleting the entry point, whose replacement is a separate concern.
    let entry = index.enter_point.load()[0];
    let deleted: Vec<_> = data
        .iter()
        .filter(|key| get_node_id(&key.0) != entry)
        .take(16)
        .cloned()
        .collect();
    assert_eq!(index.delete(&deleted).unwrap(), deleted.len());
    assert_shared(&index);
    assert_eq!(index.nodes.pin().len(), data.len() - deleted.len());
    for key in deleted {
        assert!(!index.nodes.pin().contains_key(&get_node_id(&key.0)));
    }
}

#[test]
fn collector_candidate_preserves_search_results() {
    let data = fixture();
    for metric in [
        LinearAlgorithm::EuclideanDistance,
        LinearAlgorithm::CosineSimilarity,
        LinearAlgorithm::DotProductSimilarity,
    ] {
        let control = HNSW::new(metric, 100, 4, 8, false, false);
        let candidate = candidate(metric);
        assert!(control.collector.is_none());
        control.insert(&data).unwrap();
        candidate.insert(&data).unwrap();
        for query in &data[..8] {
            let n = NonZeroUsize::new(10).unwrap();
            assert_eq!(
                control
                    .n_nearest_with_ef(query.as_slice(), n, None, NonZeroUsize::new(128))
                    .unwrap(),
                candidate
                    .n_nearest_with_ef(query.as_slice(), n, None, NonZeroUsize::new(128))
                    .unwrap()
            );
        }
    }
}

#[test]
fn dropping_candidate_releases_collector() {
    let index = candidate(LinearAlgorithm::EuclideanDistance);
    let weak = Arc::downgrade(index.collector.as_ref().unwrap());
    index.insert(&fixture()).unwrap();
    drop(index);
    assert!(
        weak.upgrade().is_none(),
        "index drop retained the shared collector"
    );
}

#[cfg(all(feature = "serde", feature = "bench-hnsw-collector-sharing"))]
#[test]
fn restored_index_gets_one_fresh_collector() {
    let index = candidate(LinearAlgorithm::EuclideanDistance);
    let data = fixture();
    index.insert(&data).unwrap();
    let bytes = serde_json::to_vec(&index).unwrap();
    let restored: HNSW<LinearAlgorithm> = serde_json::from_slice(&bytes).unwrap();
    assert_shared(&restored);
    assert!(!Arc::ptr_eq(
        index.collector.as_ref().unwrap(),
        restored.collector.as_ref().unwrap()
    ));
    assert_eq!(restored.nodes.pin().len(), index.nodes.pin().len());
    let n = NonZeroUsize::new(10).unwrap();
    assert_eq!(
        index.n_nearest(data[0].as_slice(), n, None).unwrap(),
        restored.n_nearest(data[0].as_slice(), n, None).unwrap()
    );
    restored.insert(&[EmbeddingKey::new(vec![3.0; 8])]).unwrap();
    assert_shared(&restored);
}

#[test]
fn dropping_index_after_worker_threads_releases_collector() {
    let index = candidate(LinearAlgorithm::EuclideanDistance);
    let data = fixture();
    // Keep writes serial, but exercise collectors from distinct worker threads.
    std::thread::scope(|scope| {
        for chunk in data.chunks(32) {
            let index = &index;
            scope
                .spawn(move || index.insert(chunk).unwrap())
                .join()
                .unwrap();
        }
    });
    let weak = Arc::downgrade(index.collector.as_ref().unwrap());
    drop(index);
    assert!(weak.upgrade().is_none());
}

#[test]
fn node_snapshot_remains_independent_after_index_drop() {
    let index = candidate(LinearAlgorithm::EuclideanDistance);
    let data = fixture();
    index.insert(&data).unwrap();
    let snapshot = index.get_node(&get_node_id(&data[0].0)).unwrap();
    assert!(!std::ptr::eq(
        snapshot.neighbours.guard().collector(),
        index.collector.as_ref().unwrap().as_ref()
    ));
    let weak = Arc::downgrade(index.collector.as_ref().unwrap());
    drop(index);
    assert!(weak.upgrade().is_none());
    assert_eq!(snapshot.value, data[0]);
    assert!(!snapshot.neighbours.pin().is_empty());
}

#[test]
fn candidate_supports_concurrent_reads_after_loading() {
    let index = candidate(LinearAlgorithm::EuclideanDistance);
    let data = fixture();
    index.insert(&data).unwrap();
    let n = NonZeroUsize::new(10).unwrap();
    let expected = index.n_nearest(data[0].as_slice(), n, None).unwrap();
    std::thread::scope(|scope| {
        for _ in 0..4 {
            let index = &index;
            let query = data[0].as_slice();
            let expected = &expected;
            scope.spawn(move || assert_eq!(&index.n_nearest(query, n, None).unwrap(), expected));
        }
    });
    assert_shared(&index);
}

#[test]
fn collector_benchmark_delete_all_nodes_preserves_empty_index() {
    let data = fixture();
    for index in [
        HNSW::new(LinearAlgorithm::CosineSimilarity, 500, 15, 30, false, false),
        HNSW::new_candidate(LinearAlgorithm::CosineSimilarity, 500, 15, 30, false, false),
    ] {
        assert_eq!(index.collector_sharing_bench_counts().0, 0);
        index.insert(&data).unwrap();
        assert_eq!(index.collector_sharing_bench_counts().0, data.len());
        assert_eq!(index.delete(&data).unwrap(), data.len());
        assert_eq!(index.collector_sharing_bench_counts().0, 0);
    }
}
