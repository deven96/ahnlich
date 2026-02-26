use ahnlich_similarity::LinearAlgorithm;
use ahnlich_similarity::hnsw::index::HNSW;
use ahnlich_similarity::hnsw::{HNSWConfig, Node};
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use criterion::{BatchSize, BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

// ─── Dataset Loading ───────────────────────────────────────────────────────────

pub const DATASET_PATH: &str = "sift/";

pub struct AnnDataset {
    pub sift_data: Vec<Vec<f32>>,
    pub ground_truth: Vec<Vec<i32>>,
    pub sift_query: Vec<Vec<f32>>,
}

pub fn load_dataset() -> AnnDataset {
    let dataset_location = Path::new(DATASET_PATH);

    let file_path = dataset_location.join("siftsmall_base.fvecs");
    let sift_data = read_fvec_file(&file_path);

    let ground_truth = dataset_location.join("siftsmall_groundtruth.ivecs");
    let ground_truth_data = read_ivec_file(&ground_truth);

    let sift_query = dataset_location.join("siftsmall_query.fvecs");
    let sift_query_data = read_fvec_file(&sift_query);

    AnnDataset {
        sift_data,
        ground_truth: ground_truth_data,
        sift_query: sift_query_data,
    }
}

pub fn read_fvec_file(file_path: &PathBuf) -> Vec<Vec<f32>> {
    let mut fvec_header_dim = [0u8; 4];
    let mut dataset = Vec::new();
    let opened_file = File::open(file_path).expect("Failed to open file");
    let mut reader = BufReader::new(opened_file);

    while reader.read_exact(&mut fvec_header_dim).is_ok() {
        let dimension = i32::from_le_bytes(fvec_header_dim) as usize;
        let mut vec_data = vec![0u8; dimension * 4];
        reader
            .read_exact(&mut vec_data)
            .expect("Failed to read vector");
        let vector: Vec<f32> = vec_data
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
            .collect();
        dataset.push(vector);
    }
    dataset
}

pub fn read_ivec_file(file_path: &PathBuf) -> Vec<Vec<i32>> {
    let mut header = [0u8; 4];
    let mut dataset = Vec::new();
    let file = File::open(file_path).expect("Failed to open file");
    let mut reader = BufReader::new(file);

    while reader.read_exact(&mut header).is_ok() {
        let dimension = i32::from_le_bytes(header) as usize;
        let mut vec_data = vec![0u8; dimension * 4];
        reader
            .read_exact(&mut vec_data)
            .expect("Failed to read vector");
        let vector = vec_data
            .chunks_exact(4)
            .map(|chunk| i32::from_le_bytes(chunk.try_into().unwrap()))
            .collect();
        dataset.push(vector);
    }
    dataset
}

// ─── Helpers ────────────────────────────────────────────────────────────────────

fn default_config() -> HNSWConfig {
    HNSWConfig {
        ef_construction: 100,
        maximum_connections: 40,
        maximum_connections_zero: 80,
        extend_candidates: false,
        keep_pruned_connections: false,
    }
}

fn build_index(data: &[Vec<f32>], config: HNSWConfig) -> HNSW<LinearAlgorithm> {
    let hnsw = HNSW::new_with_config(config, LinearAlgorithm::EuclideanDistance);
    for vec in data {
        let node = Node::new(vec.clone());
        hnsw.insert_node(node).unwrap();
    }
    hnsw
}

// ─── Insert Benchmarks ─────────────────────────────────────────────────────────

/// Measures time to build an HNSW index from scratch at different dataset sizes.
/// This shows how insert cost scales with the number of vectors already in the index.
fn bench_insert_build_index(c: &mut Criterion) {
    let dataset = load_dataset();
    let config = default_config();

    let mut group = c.benchmark_group("insert/build_index");
    group.sample_size(10);

    for size in [1000, 5000, 10000] {
        group.bench_with_input(BenchmarkId::new("vectors", size), &size, |b, &size| {
            let data = &dataset.sift_data[..size];
            b.iter(|| build_index(black_box(data), config));
        });
    }
    group.finish();
}

/// Measures the latency of inserting a single new vector into a pre-built 10k index.
/// This represents the steady-state incremental insert cost.
fn bench_insert_single(c: &mut Criterion) {
    let dataset = load_dataset();
    let config = default_config();

    c.bench_function("insert/single_into_10k", |b| {
        b.iter_batched(
            || {
                let hnsw = build_index(&dataset.sift_data, config);
                let query_vec = dataset.sift_query[0].clone();
                (hnsw, query_vec)
            },
            |(hnsw, query_vec)| {
                let node = Node::new(black_box(query_vec));
                hnsw.insert_node(node).unwrap();
            },
            BatchSize::LargeInput,
        );
    });
}

// ─── Search Benchmarks ──────────────────────────────────────────────────────────

/// Measures knn_search latency as the number of requested neighbors (k) increases.
/// Uses a fixed ef=100 to isolate the effect of k on performance.
///
///   k=1  → single nearest neighbor (fastest)
///   k=10 → typical use case
///   k=50 → large result set
fn bench_search_varying_k(c: &mut Criterion) {
    let dataset = load_dataset();
    let config = default_config();
    let hnsw = build_index(&dataset.sift_data, config);
    let query = Node::new(dataset.sift_query[0].clone());

    let mut group = c.benchmark_group("search/varying_k");
    for k in [1, 10, 50] {
        group.bench_with_input(BenchmarkId::new("k", k), &k, |b, &k| {
            b.iter(|| {
                hnsw.knn_search(black_box(&query), k, Some(100)).unwrap();
            });
        });
    }
    group.finish();
}

/// Measures knn_search latency as the search width parameter (ef) increases.
/// Uses a fixed k=10 to isolate the effect of ef on performance.
///
/// Higher ef explores more candidates → better recall but higher latency.
///   ef=16  → minimal search (fast, lower recall)
///   ef=50  → moderate
///   ef=100 → standard
///   ef=200 → thorough search (slow, higher recall)
fn bench_search_varying_ef(c: &mut Criterion) {
    let dataset = load_dataset();
    let config = default_config();
    let hnsw = build_index(&dataset.sift_data, config);
    let query = Node::new(dataset.sift_query[0].clone());

    let mut group = c.benchmark_group("search/varying_ef");
    for ef in [16, 50, 100, 200] {
        group.bench_with_input(BenchmarkId::new("ef", ef), &ef, |b, &ef| {
            b.iter(|| {
                hnsw.knn_search(black_box(&query), 10, Some(ef)).unwrap();
            });
        });
    }
    group.finish();
}

/// Measures total latency of running all 100 SIFT query vectors sequentially.
/// Represents batch query workload throughput.
fn bench_search_batch_queries(c: &mut Criterion) {
    let dataset = load_dataset();
    let config = default_config();
    let hnsw = build_index(&dataset.sift_data, config);
    let queries: Vec<Node> = dataset
        .sift_query
        .iter()
        .map(|v| Node::new(v.clone()))
        .collect();

    c.bench_function("search/batch_100_queries_k10_ef100", |b| {
        b.iter(|| {
            for query in &queries {
                hnsw.knn_search(black_box(query), 10, Some(100)).unwrap();
            }
        });
    });
}

// ─── Concurrent Search Benchmarks ───────────────────────────────────────────────

use std::sync::Arc;
use std::thread;

/// Measures knn_search throughput under concurrent load.
/// Multiple threads search the same pre-built index simultaneously.
/// This benchmarks the concurrent read performance of the papaya HashMap-backed HNSW.
fn bench_concurrent_search(c: &mut Criterion) {
    let dataset = load_dataset();
    let config = default_config();
    let hnsw = Arc::new(build_index(&dataset.sift_data, config));
    let queries: Vec<Node> = dataset
        .sift_query
        .iter()
        .map(|v| Node::new(v.clone()))
        .collect();

    let mut group = c.benchmark_group("search/concurrent");
    for num_threads in [1, 2, 4, 8] {
        group.bench_with_input(
            BenchmarkId::new("threads", num_threads),
            &num_threads,
            |b, &num_threads| {
                b.iter(|| {
                    let handles: Vec<_> = (0..num_threads)
                        .map(|t| {
                            let hnsw = Arc::clone(&hnsw);
                            let query = queries[t % queries.len()].clone();
                            thread::spawn(move || {
                                hnsw.knn_search(&query, 10, Some(100)).unwrap();
                            })
                        })
                        .collect();
                    for h in handles {
                        h.join().unwrap();
                    }
                });
            },
        );
    }
    group.finish();
}

// ─── Registration ───────────────────────────────────────────────────────────────

criterion_group!(
    benches,
    bench_insert_build_index,
    bench_insert_single,
    bench_search_varying_k,
    bench_search_varying_ef,
    bench_search_batch_queries,
    bench_concurrent_search,
);
criterion_main!(benches);
