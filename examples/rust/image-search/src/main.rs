use std::{
    collections::{HashMap, HashSet},
    fs::{self, File},
    io::{self, Read, Write},
    num::NonZero,
};

use ahnlich_client_rs::ai::AIClient;
use ahnlich_types::{
    ai::{AIModel, AIServerResponse, PreprocessAction},
    keyval::{StoreInput, StoreName},
    metadata::{MetadataKey, MetadataValue},
    similarity::Algorithm,
};
use clap::{Parser, Subcommand};
use tokio;

#[derive(Parser)]
#[command(name = "image-search")]
#[command(about = "An example app to index images and query indexed images via text", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run the index mode
    Index,
    /// Run the query mode
    Query,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Index => {
            println!("Running in 'index' mode");
            // You can put your indexing logic here.
            index_mode().await;
        }
        Commands::Query => {
            println!("Running in 'query' mode");
            // You can put your querying logic here.
            query_mode().await;
        }
    }
}

async fn index_mode() {
    println!("Indexing data from images");
    let ai_client = AIClient::new("127.0.0.1".to_string(), 1370)
        .await
        .expect("Could not initialize client");
    let storename = StoreName("image-search".to_string());
    let mut inputs = Vec::new();
    for entry in fs::read_dir("./images").expect("Could not read images sub-dir") {
        let path = entry.unwrap().path();
        if path.is_file() {
            let mut file = File::open(&path).expect("Could not open file");
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)
                .expect("Could not read file contents");
            inputs.push((
                StoreInput::Image(contents),
                HashMap::from_iter([(
                    MetadataKey::new("filename".to_string()),
                    MetadataValue::RawString(format!("{:?}", path)),
                )]),
            ));
        }
    }
    ai_client
        .create_store(
            storename.clone(),
            AIModel::ClipVitB32Text,
            AIModel::ClipVitB32Image,
            HashSet::new(),
            HashSet::new(),
            false,
            true,
            None,
        )
        .await
        .expect("Could not create store");
    let res = ai_client
        .set(
            storename.clone(),
            inputs,
            PreprocessAction::ModelPreprocessing,
            None,
        )
        .await
        .expect("Could not set in store");
    // Simulated async work.
    println!("Indexing complete! {:?}", res);
}

async fn query_mode() {
    // Simulated async work.
    let ai_client = AIClient::new("127.0.0.1".to_string(), 1370)
        .await
        .expect("Could not initialize client");
    let storename = StoreName("image-search".to_string());
    loop {
        let mut input = String::new();
        print!("Enter some text: ");
        io::stdout().flush().expect("Could not flush output"); // Ensure the prompt is displayed immediately

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        // Remove the trailing newline character (if needed)
        let input = input.trim();

        let res = ai_client
            .get_sim_n(
                storename.clone(),
                StoreInput::RawString(input.to_owned()),
                None,
                NonZero::new(1).unwrap(),
                Algorithm::CosineSimilarity,
                None,
            )
            .await
            .expect("Could not set in store");
        if let AIServerResponse::GetSimN(inner) = res {
            match inner.as_slice() {
                [(Some(StoreInput::Image(image_bytes)), metadata, _)] => {
                    let img =
                        image::load_from_memory(&image_bytes).expect("Could not load image bytes");
                    println!(
                        "Query results: [Closest match '{:?}']",
                        metadata
                            .get(&MetadataKey::new("filename".to_string()))
                            .unwrap()
                    );
                    let conf = viuer::Config {
                        absolute_offset: false,
                        ..Default::default()
                    };

                    viuer::print(&img, &conf).unwrap();
                }
                _ => {
                    println!("Unexpected response!")
                }
            }
        } else {
            println!("Unexpected error!")
        }
    }
}
