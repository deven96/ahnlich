use super::*;
use papaya::Guard;
use rand::{Rng, SeedableRng, rngs::StdRng};

fn fixture() -> Vec<EmbeddingKey> {
    let mut rng = StdRng::seed_from_u64(424);
    (0..128)
        .map(|_| EmbeddingKey::new((0..8).map(|_| rng.gen_range(-2.0..2.0)).collect()))
        .collect()
}

fn with_shared_collector(metric: LinearAlgorithm) -> HNSW<LinearAlgorithm> {
    HNSW::new_with_shared_collector(metric, 100, 4, 8, false, false)
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
    let index = with_shared_collector(LinearAlgorithm::EuclideanDistance);
    let separate = with_shared_collector(LinearAlgorithm::EuclideanDistance);
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
fn shared_collector_preserves_search_results() {
    let data = fixture();
    for metric in [
        LinearAlgorithm::EuclideanDistance,
        LinearAlgorithm::CosineSimilarity,
        LinearAlgorithm::DotProductSimilarity,
    ] {
        let control = HNSW::new(metric, 100, 4, 8, false, false);
        let with_shared_collector = with_shared_collector(metric);
        assert!(control.collector.is_none());
        control.insert(&data).unwrap();
        with_shared_collector.insert(&data).unwrap();
        for query in &data[..8] {
            let n = NonZeroUsize::new(10).unwrap();
            assert_eq!(
                control
                    .n_nearest_with_ef(query.as_slice(), n, None, NonZeroUsize::new(128))
                    .unwrap(),
                with_shared_collector
                    .n_nearest_with_ef(query.as_slice(), n, None, NonZeroUsize::new(128))
                    .unwrap()
            );
        }
    }
}

#[test]
fn dropping_index_with_shared_collector_releases_collector() {
    let index = with_shared_collector(LinearAlgorithm::EuclideanDistance);
    let weak = Arc::downgrade(index.collector.as_ref().unwrap());
    index.insert(&fixture()).unwrap();
    drop(index);
    assert!(
        weak.upgrade().is_none(),
        "index drop retained the shared collector"
    );
}

#[cfg(feature = "serde")]
#[test]
fn restored_index_gets_one_fresh_collector() {
    let index = with_shared_collector(LinearAlgorithm::EuclideanDistance);
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
    let index = with_shared_collector(LinearAlgorithm::EuclideanDistance);
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
    let index = with_shared_collector(LinearAlgorithm::EuclideanDistance);
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
fn shared_collector_supports_concurrent_reads_after_loading() {
    let index = with_shared_collector(LinearAlgorithm::EuclideanDistance);
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

#[cfg(feature = "bench-experiments")]
#[test]
fn collector_benchmark_delete_all_nodes_preserves_empty_index() {
    let data = fixture();
    for index in [
        HNSW::new(LinearAlgorithm::CosineSimilarity, 500, 15, 30, false, false),
        HNSW::new_with_shared_collector(
            LinearAlgorithm::CosineSimilarity,
            500,
            15,
            30,
            false,
            false,
        ),
    ] {
        assert_eq!(index.collector_sharing_bench_counts().0, 0);
        index.insert(&data).unwrap();
        assert_eq!(index.collector_sharing_bench_counts().0, data.len());
        assert_eq!(index.delete(&data).unwrap(), data.len());
        assert_eq!(index.collector_sharing_bench_counts().0, 0);
    }
}

#[test]
fn counter_shards_preserve_graph_results_and_collector_lifecycle() {
    let data = fixture();
    for metric in [
        LinearAlgorithm::EuclideanDistance,
        LinearAlgorithm::CosineSimilarity,
        LinearAlgorithm::DotProductSimilarity,
    ] {
        for shards in [1, 2] {
            let config = HNSWConfig {
                ef_construction: 100,
                maximum_connections: 4,
                maximum_connections_zero: 8,
                extend_candidates: false,
                keep_pruned_connections: false,
            };
            let control = HNSW::new_with_config_and_shared_collector(config, metric);
            let candidate = HNSW::new_with_config_and_shared_collector_and_counter_shards(
                config,
                metric,
                NonZeroUsize::new(shards).unwrap(),
            );
            assert!(control.node_counter_shards.is_none());
            assert_eq!(candidate.node_counter_shards.unwrap().get(), shards);
            control.insert(&data).unwrap();
            candidate.insert(&data).unwrap();
            // Public nodes with independent collections must be rebound too.
            let extra = EmbeddingKey::new(vec![3.0; 8]);
            control.insert_node(Node::new(extra.clone())).unwrap();
            candidate.insert_node(Node::new(extra.clone())).unwrap();
            assert_shared(&candidate);
            for query in data.iter().take(8) {
                assert_eq!(
                    control
                        .n_nearest(query.as_slice(), NonZeroUsize::new(10).unwrap(), None)
                        .unwrap(),
                    candidate
                        .n_nearest(query.as_slice(), NonZeroUsize::new(10).unwrap(), None)
                        .unwrap(),
                );
            }
            let mut deleted = data.clone();
            deleted.push(extra);
            assert_eq!(candidate.delete(&deleted).unwrap(), deleted.len());
            assert!(candidate.nodes.pin().is_empty());
            let weak = Arc::downgrade(candidate.collector.as_ref().unwrap());
            drop(candidate);
            assert!(weak.upgrade().is_none());
        }
    }
}

#[cfg(feature = "serde")]
#[test]
fn counter_shards_restore_policy_without_changing_snapshot_format() {
    let index = HNSW::new_with_shared_collector_and_counter_shards(
        LinearAlgorithm::EuclideanDistance,
        100,
        4,
        8,
        false,
        false,
    );
    let data = fixture();
    index.insert(&data).unwrap();
    let bytes = serde_json::to_vec(&index).unwrap();
    let json: serde_json::Value = serde_json::from_slice(&bytes).unwrap();
    assert!(json.get("node_counter_shards").is_none());
    let restored: HNSW<LinearAlgorithm> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(restored.node_counter_shards.unwrap().get(), 1);
    assert_shared(&restored);
    assert_eq!(
        index
            .n_nearest(data[0].as_slice(), NonZeroUsize::new(10).unwrap(), None)
            .unwrap(),
        restored
            .n_nearest(data[0].as_slice(), NonZeroUsize::new(10).unwrap(), None)
            .unwrap(),
    );
    restored.insert(&[EmbeddingKey::new(vec![3.0; 8])]).unwrap();
    assert_shared(&restored);
}

#[cfg(feature = "serde")]
#[test]
fn configured_counter_shards_apply_to_creation_and_restore() {
    // Isolate startup-global configuration from other tests in this process.
    const CHILD: &str = "AHNLICH_TEST_COUNTER_SHARD_POLICY";
    if std::env::var_os(CHILD).is_none() {
        let output = std::process::Command::new(std::env::current_exe().unwrap())
            .args(["--exact", "hnsw::index::collector_sharing_tests::configured_counter_shards_apply_to_creation_and_restore"])
            .env(CHILD, "1")
            .output()
            .unwrap();
        assert!(
            output.status.success(),
            "{}\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
        return;
    }
    configure_node_counter_shards(NonZeroUsize::new(4).unwrap());
    let index = HNSW::new_with_shared_collector_and_counter_shards(
        LinearAlgorithm::EuclideanDistance,
        100,
        4,
        8,
        false,
        false,
    );
    assert_eq!(index.node_counter_shards.unwrap().get(), 4);
    let data = fixture();
    index.insert(&data).unwrap();
    let bytes = serde_json::to_vec(&index).unwrap();
    let restored: HNSW<LinearAlgorithm> = serde_json::from_slice(&bytes).unwrap();
    assert_eq!(restored.node_counter_shards.unwrap().get(), 4);
    assert_shared(&restored);
    restored.insert(&[EmbeddingKey::new(vec![3.0; 8])]).unwrap();
    assert_shared(&restored);
    assert_eq!(
        index
            .n_nearest(data[0].as_slice(), NonZeroUsize::new(10).unwrap(), None)
            .unwrap(),
        restored
            .n_nearest(data[0].as_slice(), NonZeroUsize::new(10).unwrap(), None)
            .unwrap(),
    );
}
