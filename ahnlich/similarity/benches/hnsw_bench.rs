use ahnlich_similarity::LinearAlgorithm;
use ahnlich_similarity::hnsw::Node;
use ahnlich_similarity::hnsw::index::HNSW;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use criterion::{BatchSize, Criterion, black_box, criterion_group, criterion_main};

pub const DATASET_PATH: &str = "sift/";

pub struct AnnDataset {
    pub sift_data: Vec<Vec<f32>>,
    pub ground_truth: Vec<Vec<i32>>,
    pub sift_query: Vec<Vec<f32>>,
}

pub fn load_dataset() -> AnnDataset {
    //change to sift
    let dataset_location = Path::new(DATASET_PATH);

    let file_path = dataset_location.join("siftsmall_base.fvecs");
    let sift_data = read_fvec_file(&file_path);

    // sift_ground_truth
    let ground_truth = dataset_location.join("siftsmall_groundtruth.ivecs");
    let ground_truth_data = read_ivec_file(&ground_truth);
    // sift_query.fvecs
    let sift_query = dataset_location.join("siftsmall_query.fvecs");

    let sift_query_data = read_fvec_file(&sift_query);

    AnnDataset {
        sift_data,
        ground_truth: ground_truth_data,
        sift_query: sift_query_data,
    }
}

pub fn read_fvec_file(file_path: &PathBuf) -> Vec<Vec<f32>> {
    println!("Reading fvec file {:?}", file_path);

    let mut fvec_header_dim = [0u8; 4];

    let mut dataset = Vec::new();
    let opened_file = File::open(file_path).expect("Failed to open file");

    let mut reader = BufReader::new(opened_file);

    while reader.read_exact(&mut fvec_header_dim).is_ok() {
        let dimension = i32::from_le_bytes(fvec_header_dim) as usize;

        // Prepare a buffer for d * 4 bytes
        let mut vec_data = vec![0u8; dimension * 4];
        reader
            .read_exact(&mut vec_data)
            .expect("Failed to read vector");

        // Convert the byte chunk into f32 values
        let vector: Vec<f32> = vec_data
            .chunks_exact(4)
            .map(|chunk| f32::from_le_bytes(chunk.try_into().unwrap()))
            .collect();

        dataset.push(vector);
    }

    dataset
}

pub fn read_ivec_file(file_path: &PathBuf) -> Vec<Vec<i32>> {
    println!("Reading Ivec file {:?}", file_path);

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
                LinearAlgorithm::EuclideanDistance,
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

    c.bench_function("hnsw_incremental_insert", |b| {
        b.iter_batched(
            || {
                // Setup (not measured)
                let mut hnsw = HNSW::new(
                    config.ef_construction,
                    config.maximum_connections,
                    config.maximum_connections_zero,
                    LinearAlgorithm::EuclideanDistance,
                );

                for vec in &dataset.sift_data {
                    let node = Node::new(vec.clone());
                    hnsw.insert(node).unwrap();
                }

                let query_vec = dataset.sift_query[0].clone();

                (hnsw, query_vec)
            },
            |(mut hnsw, query_vec)| {
                // Measured
                let node = Node::new(black_box(query_vec));
                hnsw.insert(node).unwrap();
            },
            BatchSize::SmallInput,
        )
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
        LinearAlgorithm::EuclideanDistance,
    );

    for vec in &dataset.sift_data {
        let node = Node::new(vec.clone());
        hnsw.insert(node).unwrap();
    }

    c.bench_function("hnsw_search_k10", |b| {
        let query = Node::new(dataset.sift_query[0].clone());

        b.iter(|| {
            hnsw.knn_search(
                black_box(&query),
                10, // k
                Some(config.knn_search_ef_param),
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
