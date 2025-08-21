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
let mut ai_client = AIClient::new("127.0.0.1:1370".to_string()).await?;
let tracing_id = None;

// create a store for images
ai_client.create_store(CreateStoreAI {
    store: "image_store".to_string(),
    index_model: AIModel::AllMiniLML6V2,
    query_model: AIModel::AllMiniLML6V2,
    predicates: Some(HashSet::from(["tag".to_string()])),
    store_original: Some(true),
    error_if_exists: Some(true),
}, tracing_id.clone()).await?;

// ingest image bytes with metadata
ai_client.set(SetAI {
    store: "image_store".to_string(),
    inputs: vec![
      (StoreInput::RawBytes(img_bytes), HashMap::from([("tag".to_string(), "nature")]))
    ],
    preprocess_action: Some(PreprocessAction::NoPreprocessing),
}, tracing_id.clone()).await?;
```

## ➕ Try it yourself

- Clone the example repository  
- Launch `ahnlich-db` and `ahnlich-ai` locally via Docker or binaries  
- Modify the image folder path and metadata tags  
- Build and run the Rust example app to visualize results
