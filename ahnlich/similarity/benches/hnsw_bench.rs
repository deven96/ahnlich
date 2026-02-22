use ahnlich_similarity::hnsw::index::HNSW;
use ahnlich_similarity::hnsw::{LayerIndex, Node};

use ahnlich_similarity::tests::datasets::loader::load_dataset;
use criterion::{Criterion, criterion_group, criterion_main};

#[derive(Clone, Copy)]
struct HNSWConfig {
    ef_construction: usize,
    maximum_connections: usize,
    maximum_connections_zero: Option<usize>,
    knn_search_ef_param: usize,
}

fn bench_hnsw_insert(c: &mut Criterion) {
    let dataset = load_dataset();

    let config = HNSWConfig {
        ef_construction: 100,
        maximum_connections: 40,
        maximum_connections_zero: Some(80),
        knn_search_ef_param: 16,
    };

    c.bench_function("hnsw_insert_sift10k", |b| {
        b.iter(|| {
            let mut hnsw = HNSW::new(
                config.ef_construction,
                config.maximum_connections,
                config.maximum_connections_zero,
            );

            for vec in &dataset.sift_data {
                let node = Node::new(vec.clone());
                hnsw.insert(node).unwrap();
            }
        })
    });
}

fn bench_hnsw_incremental_insert(c: &mut Criterion) {
    let dataset = load_dataset();

    let config = HNSWConfig {
        ef_construction: 100,
        maximum_connections: 40,
        maximum_connections_zero: Some(80),
        knn_search_ef_param: 16,
    };

    // Build index once (NOT measured)
    let mut hnsw = HNSW::new(
        config.ef_construction,
        config.maximum_connections,
        config.maximum_connections_zero,
    );

    for vec in &dataset.sift_data {
        let node = Node::new(vec.clone());
        hnsw.insert(node).unwrap();
    }

    // Use query vectors as new inserts
    let mut insert_iter = dataset.sift_query.iter();

    c.bench_function("hnsw_incremental_insert", |b| {
        b.iter(|| {
            if let Some(vec) = insert_iter.next() {
                let node = Node::new(vec.clone());
                hnsw.insert(node).unwrap();
            }
        })
    });
}

fn bench_search_layer(c: &mut Criterion) {
    let dataset = load_dataset();

    let config = HNSWConfig {
        ef_construction: 100,
        maximum_connections: 40,
        maximum_connections_zero: Some(80),
        knn_search_ef_param: 32,
    };

    // Build index once
    let mut hnsw = HNSW::new(
        config.ef_construction,
        config.maximum_connections,
        config.maximum_connections_zero,
    );

    for vec in &dataset.sift_data {
        let node = Node::new(vec.clone());
        hnsw.insert(node).unwrap();
    }

    // Prepare query + entry point
    let query = Node::new(dataset.sift_query[0].clone());
    let entry_points = hnsw.current_enterpoint().to_vec();

    c.bench_function("search_layer_level_0", |b| {
        b.iter(|| {
            hnsw.search_layer(
                &query,
                &entry_points,
                100, // ef
                &LayerIndex(0),
            )
            .unwrap();
        })
    });
}

criterion_group!(
    benches,
    bench_hnsw_insert,
    bench_hnsw_incremental_insert,
    bench_search_layer
);

criterion_main!(benches);
