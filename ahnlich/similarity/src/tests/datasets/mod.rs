use std::io::Read;
use std::{fs::File, io::BufReader, path::PathBuf};

pub mod download;
pub mod loader;

pub struct AnnDataset {
    pub sift_data: Vec<Vec<f32>>,
    pub ground_truth: Vec<Vec<i32>>,
    pub sift_query: Vec<Vec<f32>>,
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
            .filter_map(|chunk| {
                if let Ok(valid_chunk) = chunk.try_into() {
                    Some(f32::from_le_bytes(valid_chunk))
                } else {
                    None
                }
            })
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
