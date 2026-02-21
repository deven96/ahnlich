use flate2::bufread::GzDecoder;

use std::{
    fs::File,
    io::BufReader,
    path::{Path, PathBuf},
};

use crate::tests::datasets::{
    AnnDataset,
    download::{DATASET_NAME, DATASET_PATH, download_dataset},
    read_fvec_file, read_ivec_file,
};

pub fn decompress_tar(file_path: &str) -> PathBuf {
    let tar_gz = BufReader::new(File::open(file_path).expect("Failed to open tar gz file"));
    let tar = GzDecoder::new(tar_gz);
    let mut archive = tar::Archive::new(tar);

    archive.unpack(DATASET_PATH).expect("failed to unpack file");

    Path::new(file_path).join("sift")
}

pub fn load_dataset() -> AnnDataset {
    let path_str = format!("{DATASET_PATH}{DATASET_NAME}");

    if !Path::new(&path_str).exists() {
        download_dataset();
        decompress_tar(&path_str);
    }

    println!("Loading Datasets ...");

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
