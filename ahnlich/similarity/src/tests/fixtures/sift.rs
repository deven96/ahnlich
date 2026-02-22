use std::io::Read;
use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

pub const DATASET_PATH: &str = ".datasets/";

pub struct AnnDataset {
    pub sift_data: Vec<Vec<f32>>,
    pub ground_truth: Vec<Vec<i32>>,
    pub sift_query: Vec<Vec<f32>>,
}

pub fn load_dataset() -> AnnDataset {
    //change to sift
    let dataset_location = Path::new(DATASET_PATH).join("siftsmall");

    let file_path = dataset_location.clone().join("siftsmall_base.fvecs");
    let sift_data = read_fvec_file(&file_path);

    // sift_ground_truth
    let ground_truth = dataset_location.clone().join("siftsmall_groundtruth.ivecs");
    let ground_truth_data = read_ivec_file(&ground_truth);
    // sift_query.fvecs
    let sift_query = dataset_location.clone().join("siftsmall_query.fvecs");

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
