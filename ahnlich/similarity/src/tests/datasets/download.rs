use ftp::FtpStream;
use std::{
    fs::{self, File},
    io::Write,
    path::Path,
};

pub const DATASET_PATH: &str = ".datasets/";
pub const DATASET_NAME: &str = "siftsmall.tar.gz";
//pub const DATASET_NAME_2: &str = "sift.tar.gz";

pub const BASE_URL: &str = "ftp.irisa.fr";

//pub const DATASET_URL: &str = "ftp://ftp.irisa.fr/local/texmex/corpus/siftsmall.tar.gz";
//pub const DATASET_URL_2: &str = "ftp://ftp.irisa.fr/local/texmex/corpus/sift.tar.gz";

pub fn download_dataset() {
    println!("Downloading Datasets ...");
    // Create a connection to an FTP server and authenticate to it.
    let mut ftp_stream = FtpStream::connect(format!("{}:21", BASE_URL)).unwrap();
    ftp_stream.login("anonymous", "anonymous").unwrap();

    let download_path = Path::new(DATASET_PATH);

    if !download_path.exists() {
        fs::create_dir_all(download_path).expect("Failed to create download path");
    }

    // navigate into the path from the URL
    ftp_stream
        .cwd("/local/texmex/corpus")
        .expect("Failed to navigate ftp server");

    // Get the current directory that the client will be reading from and writing to.
    println!("Current directory: {}", ftp_stream.pwd().unwrap());

    // retrieve the file
    // change back to sift
    let data = ftp_stream
        .simple_retr("siftsmall.tar.gz")
        .expect("Failed to retrieve")
        .into_inner();

    let dataset_path = format!("{DATASET_PATH}{DATASET_NAME}");
    let mut file = File::create(dataset_path).expect("Failed to create file");
    file.write_all(&data).expect("Failed to write buffer");

    ftp_stream.quit().expect("Failed to quit ftp");
}
