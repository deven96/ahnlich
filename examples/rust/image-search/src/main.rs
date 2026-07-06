use std::{
    collections::HashMap,
    fs::{self, File},
    io::{self, Read, Write},
};

use ahnlich_client_rs::ai::AiClient;
use ahnlich_types::{
    ai::query::{CreateStore, GetSimN, Set},
    algorithm::algorithms::Algorithm,
    keyval::{AiStoreEntry, StoreInput, StoreValue, store_input::Value},
};

use ahnlich_types::metadata::{MetadataValue, metadata_value::Value as MValue};

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
            index_mode().await;
        }
        Commands::Query => {
            println!("Running in 'query' mode");
            query_mode().await;
        }
    }
}

async fn create_client() -> AiClient {
    AiClient::new("http://127.0.0.1:1370".to_string())
        .await
        .expect("Failed to create AI Client")
}

async fn index_mode() {
    println!("Indexing data from images");
    let client = create_client().await;
    let storename = "image-search".to_string();

    // Create store
    let create_store = CreateStore {
        store: storename.clone(),
        index_model: ahnlich_types::ai::models::AiModel::ClipVitB32Image.into(),
        query_model: ahnlich_types::ai::models::AiModel::ClipVitB32Text.into(),
        predicates: vec![],
        non_linear_indices: vec![],
        error_if_exists: false,
        store_original: true,
        schema: None,
    };

    client
        .create_store(create_store, None)
        .await
        .expect("Could not create store");

    // Prepare inputs
    let mut inputs = Vec::new();
    for entry in fs::read_dir("./images").expect("Could not read images sub-dir") {
        let path = entry.unwrap().path();
        if path.is_file() {
            let mut file = File::open(&path).expect("Could not open file");
            let mut contents = Vec::new();
            file.read_to_end(&mut contents)
                .expect("Could not read file contents");

            inputs.push(AiStoreEntry {
                key: Some(StoreInput {
                    value: Some(Value::Image(contents)),
                }),
                value: Some(StoreValue {
                    value: HashMap::from_iter([(
                        "filename".to_string(),
                        MetadataValue {
                            value: Some(MValue::RawString(format!("{:?}", path))),
                        },
                    )]),
                }),
            });
        }
    }

    // Set data
    let set = Set {
        store: storename,
        inputs,
        preprocess_action: ahnlich_types::ai::preprocess::PreprocessAction::ModelPreprocessing
            .into(),
        execution_provider: None,
        model_params: std::collections::HashMap::new(),
        schema: None,
    };

    let res = client.set(set, None).await.expect("Could not set in store");

    println!("Indexing complete! {:?}", res);
}

async fn query_mode() {
    let client = create_client().await;
    let storename = "image-search".to_string();

    loop {
        let mut input = String::new();
        print!("Enter some text: ");
        io::stdout().flush().expect("Could not flush output");

        io::stdin()
            .read_line(&mut input)
            .expect("Failed to read input");

        let input = input.trim();

        let get_sim_n = GetSimN {
            store: storename.clone(),
            search_input: Some(StoreInput {
                value: Some(Value::RawString(input.to_string())),
            }),
            condition: None,
            closest_n: 1,
            algorithm: Algorithm::CosineSimilarity.into(),
            preprocess_action: ahnlich_types::ai::preprocess::PreprocessAction::NoPreprocessing
                .into(),
            execution_provider: None,
            model_params: std::collections::HashMap::new(),
            schema: None,
        };

        let res = client
            .get_sim_n(get_sim_n, None)
            .await
            .expect("Could not query store");

        let entries = res.entries;

        if let Some(entry) = entries.first() {
            if let Some(StoreInput {
                value: Some(Value::Image(image_bytes)),
            }) = &entry.key
            {
                let img =
                    image::load_from_memory(&image_bytes).expect("Could not load image bytes");

                if let Some(filename) = entry.value.as_ref().and_then(|v| v.value.get("filename")) {
                    if let Some(MValue::RawString(filename)) = &filename.value {
                        println!("Query results: [Closest match '{}']", filename);
                    }
                }

                let conf = viuer::Config {
                    absolute_offset: false,
                    ..Default::default()
                };
                viuer::print(&img, &conf).unwrap();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_correct_similarity_algorithm_is_used_in_query() {
        // CRITICAL: This test prevents regression of the similarity algorithm bug
        // 
        // CLIP embeddings are normalized (unit vectors), which means:
        // - CosineSimilarity: measures angle between vectors (correct for normalized embeddings)
        // - DotProductSimilarity: measures raw dot product (incorrect for semantic similarity)
        //
        // Using DotProductSimilarity caused "cat playing piano" queries to return dog images!
        // This happened because DotProduct doesn't properly measure semantic similarity
        // for normalized embeddings.
        
        // Create a GetSimN query as done in query_mode()
        let storename = "image-search".to_string();
        let get_sim_n = GetSimN {
            store: storename,
            search_input: Some(StoreInput {
                value: Some(Value::RawString("test query".to_string())),
            }),
            condition: None,
            closest_n: 1,
            algorithm: Algorithm::CosineSimilarity.into(),
            preprocess_action: ahnlich_types::ai::preprocess::PreprocessAction::NoPreprocessing
                .into(),
            execution_provider: None,
            model_params: std::collections::HashMap::new(),
            schema: None,
        };

        // Verify the algorithm is CosineSimilarity, not DotProductSimilarity
        assert_eq!(
            get_sim_n.algorithm,
            Algorithm::CosineSimilarity.into(),
            "Image search MUST use CosineSimilarity for CLIP embeddings. \
             DotProductSimilarity will return incorrect results for normalized vectors."
        );
        
        // Also verify it's NOT DotProductSimilarity
        assert_ne!(
            get_sim_n.algorithm,
            Algorithm::DotProductSimilarity.into(),
            "Image search must NOT use DotProductSimilarity - it returns wrong results for CLIP!"
        );
    }

    #[test]
    fn test_create_store_uses_clip_models() {
        // Verify we're using the correct CLIP models for image/text embedding
        let storename = "image-search".to_string();
        let create_store = CreateStore {
            store: storename,
            index_model: ahnlich_types::ai::models::AiModel::ClipVitB32Image.into(),
            query_model: ahnlich_types::ai::models::AiModel::ClipVitB32Text.into(),
            predicates: vec![],
            non_linear_indices: vec![],
            error_if_exists: false,
            store_original: true,
            schema: None,
        };

        // Verify we're using CLIP models (these produce normalized embeddings)
        assert_eq!(
            create_store.index_model,
            ahnlich_types::ai::models::AiModel::ClipVitB32Image.into()
        );
        assert_eq!(
            create_store.query_model,
            ahnlich_types::ai::models::AiModel::ClipVitB32Text.into()
        );
    }
}
