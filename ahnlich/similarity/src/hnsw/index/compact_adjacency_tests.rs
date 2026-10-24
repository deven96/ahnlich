use super::*;
use rand::{Rng, SeedableRng, rngs::StdRng};

use super::experiments::ControlHNSW;

type Candidate = HNSW<LinearAlgorithm, CompactEdges>;

fn data(count: usize) -> Vec<EmbeddingKey> {
    let mut rng = StdRng::seed_from_u64(444);
    (0..count)
        .map(|_| EmbeddingKey::new((0..8).map(|_| rng.gen_range(-2.0..2.0)).collect()))
        .collect()
}

fn topology<E: EdgeStorage>(index: &HNSW<LinearAlgorithm, E>) -> serde_json::Value {
    let mut temp = TempHNSW::from(index);
    for ids in temp.graph.values_mut() {
        ids.sort_unstable();
    }
    for node in temp.nodes.values_mut() {
        for ids in node.neighbours.values_mut() {
            ids.sort_unstable();
        }
        node.back_links.sort_unstable();
    }
    temp.enter_point.sort_unstable();
    serde_json::to_value(temp).unwrap()
}

fn assert_queries<A: EdgeStorage, B: EdgeStorage>(
    control: &HNSW<LinearAlgorithm, A>,
    candidate: &HNSW<LinearAlgorithm, B>,
    queries: &[EmbeddingKey],
) {
    let ids = control
        .nodes
        .pin()
        .keys()
        .map(|id| id.0)
        .collect::<Vec<_>>();
    // The large list exceeds the exact-scan threshold, exercising in-filtered graph
    // traversal. Extra IDs do not identify graph nodes, as with stale read snapshots.
    let mut large = ids
        .iter()
        .step_by(2)
        .copied()
        .collect::<std::collections::HashSet<_>>();
    let mut missing = u64::MAX;
    while large.len() <= BRUTE_FORCE_ACCEPT_LIST_THRESHOLD {
        if !ids.contains(&missing) {
            large.insert(missing);
        }
        missing -= 1;
    }
    let small = ids.iter().take(8).copied().collect();
    for accept in [
        None,
        Some(std::collections::HashSet::new()),
        Some(small),
        Some(large),
    ] {
        for query in queries {
            for ef in [None, NonZeroUsize::new(1), NonZeroUsize::new(100)] {
                assert_eq!(
                    candidate
                        .n_nearest_with_ef(
                            query.as_slice(),
                            NonZeroUsize::new(10).unwrap(),
                            accept.clone(),
                            ef
                        )
                        .unwrap(),
                    control
                        .n_nearest_with_ef(
                            query.as_slice(),
                            NonZeroUsize::new(10).unwrap(),
                            accept.clone(),
                            ef
                        )
                        .unwrap(),
                );
            }
        }
    }
}

#[test]
fn compact_graph_matches_control_and_restores_in_both_directions() {
    let data = data(160);
    for metric in [
        LinearAlgorithm::EuclideanDistance,
        LinearAlgorithm::CosineSimilarity,
        LinearAlgorithm::DotProductSimilarity,
    ] {
        let config = HNSWConfig {
            maximum_connections: 4,
            maximum_connections_zero: 8,
            ..HNSWConfig::default()
        };
        let control = ControlHNSW::new_control_with_config_and_shared_collector_and_counter_shards(
            config,
            metric,
            NonZeroUsize::new(1).unwrap(),
        );
        let candidate = Candidate::new_with_config_and_shared_collector_and_counter_shards(
            config,
            metric,
            NonZeroUsize::new(1).unwrap(),
        );
        for batch in data.chunks(40) {
            control.insert(batch).unwrap();
            candidate.insert(batch).unwrap();
        }
        // Duplicate writes remain idempotent.
        control.insert(&data[..8]).unwrap();
        candidate.insert(&data[..8]).unwrap();
        assert_eq!(topology(&control), topology(&candidate));
        assert_queries(&control, &candidate, &data[..3]);

        let restored_compact: Candidate =
            serde_json::from_str(&serde_json::to_string(&control).unwrap()).unwrap();
        let restored_control: ControlHNSW<LinearAlgorithm> =
            serde_json::from_str(&serde_json::to_string(&candidate).unwrap()).unwrap();
        assert_eq!(topology(&control), topology(&restored_compact));
        assert_eq!(topology(&control), topology(&restored_control));
        assert_queries(&control, &restored_compact, &data[..3]);
        assert_queries(&restored_control, &candidate, &data[..3]);

        let entry = candidate.enter_point.load().ids[0];
        let deleted = data
            .iter()
            .filter(|key| get_node_id(key.as_slice()) != entry)
            .take(24)
            .cloned()
            .collect::<Vec<_>>();
        assert_eq!(control.delete(&deleted).unwrap(), deleted.len());
        assert_eq!(candidate.delete(&deleted).unwrap(), deleted.len());
        assert_eq!(topology(&control), topology(&candidate));
        assert_queries(&control, &candidate, &data[40..43]);

        let collector = candidate.collector.as_ref().unwrap();
        for node in candidate.nodes.pin().values() {
            assert!(std::ptr::eq(
                node.neighbours.guard().collector(),
                collector.as_ref()
            ));
            assert!(std::ptr::eq(
                node.back_links.guard().collector(),
                collector.as_ref()
            ));
            for (layer, edges) in node.neighbours.pin().iter() {
                assert!(
                    edges.load().len()
                        <= if layer.0 == 0 {
                            config.maximum_connections_zero
                        } else {
                            config.maximum_connections
                        }
                );
            }
        }
        let weak = Arc::downgrade(collector);
        let snapshot = candidate.nodes.pin().values().next().unwrap().clone();
        drop(candidate);
        assert!(weak.upgrade().is_none());
        // A cloned node owns independent collection/edge snapshots.
        for edges in snapshot.neighbours.pin().values() {
            assert!(!edges.load().is_empty());
        }
    }
}

#[test]
fn compact_edges_preserve_concurrent_adds_and_removes() {
    let edges = CompactEdges::from_pointee(Vec::new());
    std::thread::scope(|scope| {
        for worker in 0..8 {
            let edges = &edges;
            scope.spawn(move || {
                for id in worker * 32..(worker + 1) * 32 {
                    edges.insert_edge(NodeId(id));
                    edges.insert_edge(NodeId(id));
                }
            });
        }
    });
    let snapshot = edges.load_full();
    assert_eq!(snapshot.len(), 256);
    std::thread::scope(|scope| {
        for worker in 0..8 {
            let edges = &edges;
            scope.spawn(move || {
                for id in (worker * 32..(worker + 1) * 32).step_by(2) {
                    edges.remove_edge(NodeId(id));
                }
            });
        }
    });
    let mut ids = edges.load().as_ref().clone();
    ids.sort_unstable();
    assert_eq!(
        ids,
        (0..256)
            .filter(|id| id % 2 == 1)
            .map(NodeId)
            .collect::<Vec<_>>()
    );
    assert_eq!(snapshot.len(), 256);
}

#[test]
fn compact_pruning_retries_selection_after_concurrent_add_or_remove() {
    for remove in [false, true] {
        let edges = CompactEdges::from_pointee(vec![NodeId(1), NodeId(2), NodeId(3)]);
        let barrier = std::sync::Barrier::new(2);
        let mut calls = 0;
        std::thread::scope(|scope| {
            scope.spawn(|| {
                barrier.wait();
                if remove {
                    edges.remove_edge(NodeId(3));
                } else {
                    edges.insert_edge(NodeId(99));
                }
                barrier.wait();
            });
            edges
                .prune(1, None, None, |ids| {
                    calls += 1;
                    let selected = vec![*ids.iter().max().unwrap()];
                    if calls == 1 {
                        barrier.wait();
                        barrier.wait();
                    }
                    Ok(selected)
                })
                .unwrap();
        });
        assert_eq!(calls, 2);
        assert_eq!(&**edges.load(), &[NodeId(if remove { 2 } else { 99 })]);
    }
}
