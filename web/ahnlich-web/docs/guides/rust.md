---
title: ⚡ Rust Image Search
---

**Source**: [examples/rust/image‑search](https://github.com/deven96/ahnlich/tree/main/examples/rust/image-search)

This guide demonstrates building an **image-based similarity search** application using the Rust SDK. It covers:

- Initializing `ahnlich-db` and `ahnlich-ai` stores
- Ingesting image files as raw bytes into an **AI Store**
- Running similarity queries based on image embeddings
- Using metadata (e.g. tags) for filtering results  
- Returning both vector similarity scores and original image references

## 🔧 What you’ll learn

1. Setting up an **AI Store** via Rust client.
2. Feeding images (byte streams) into the store.
3. Querying the store with a new image file to retrieve visually similar results.
4. Applying metadata filters (e.g. `tag == "nature"`).
5. Understanding handling of raw input retention for display.

## 💡 Highlighted snippet

```rust
use ahnlich_client_rs::ai::AiClient;
use ahnlich_types::ai::query::{CreateStore, Set};
use ahnlich_types::ai::models::AiModel;
use ahnlich_types::ai::preprocess::PreprocessAction;
use ahnlich_types::keyval::{AiStoreEntry, StoreInput, StoreValue};
use ahnlich_types::keyval::store_input::Value;
use ahnlich_types::metadata::{MetadataValue, metadata_value::Value as MValue};
use std::collections::HashMap;

let tracing_id = None;

let ai_client = AiClient::new("127.0.0.1:1370".to_string()).await?;

// create a store for images
ai_client.create_store(CreateStore {
    store: "image_store".to_string(),
    index_model: AiModel::ClipVitB32Image as i32,
    query_model: AiModel::ClipVitB32Text as i32,
    predicates: vec!["tag".to_string()],
    non_linear_indices: vec![],
    store_original: true,
    error_if_exists: true,
}, tracing_id.clone()).await?;

// ingest image bytes with metadata
ai_client.set(Set {
    store: "image_store".to_string(),
    inputs: vec![
        AiStoreEntry {
            key: Some(StoreInput { value: Some(Value::Image(img_bytes)) }),
            value: Some(StoreValue {
                value: HashMap::from([("tag".to_string(), MetadataValue {
                    value: Some(MValue::RawString("nature".into())),
                })]),
            }),
        },
    ],
    preprocess_action: PreprocessAction::NoPreprocessing as i32,
    execution_provider: None,
    model_params: HashMap::new(),
}, tracing_id.clone()).await?;
```

## ➕ Try it yourself

- Clone the example repository  
- Launch `ahnlich-db` and `ahnlich-ai` locally via Docker or binaries  
- Modify the image folder path and metadata tags  
- Build and run the Rust example app to visualize results
