---
title: Convert to Embeddings
sidebar_position: 20
---

# Convert to Embeddings

The `convert_to_embeddings` method converts raw inputs (text, images, audio) into embeddings using a specified AI model, without storing them in a database. This is useful for generating embeddings on-demand or testing model behavior.

## Method Signature

```rust
pub async fn convert_to_embeddings(
    &mut self,
    store_name: String,
    inputs: Vec<StoreInput>,
    preprocess_action: PreprocessAction,
    execution_provider: Option<ExecutionProvider>,
    model_params: HashMap<String, String>,
) -> Result<StoreInputToEmbeddingsList>
```

## Parameters

- `store_name`: Name of the store (determines which model to use)
- `inputs`: Vector of inputs to convert (text, images, or audio)
- `preprocess_action`: How to preprocess inputs (`NoPreprocessing` or `ModelPreprocessing`)
- `execution_provider`: Optional execution provider (CPU, CUDA, etc.)
- `model_params`: Optional model-specific parameters (e.g., `confidence_threshold` for face detection)

## Basic Example

```rust
use ahnlich_client_rs::prelude::*;
use std::collections::HashMap;

let inputs = vec![
    StoreInput {
        value: Some(store_input::Value::RawString("Hello world".to_string())),
    },
];

let response = client
    .convert_to_embeddings(
        "my_store".to_string(),
        inputs,
        PreprocessAction::NoPreprocessing,
        None,
        HashMap::new(),
    )
    .await?;

// Access embeddings
for item in response.values {
    if let Some(Variant::Single(embedding_with_meta)) = item.variant {
        if let Some(embedding) = embedding_with_meta.embedding {
            println!("Embedding dimensions: {}", embedding.key.len());
        }
    }
}
```

## Face Detection with Metadata (Buffalo-L / SFace)

Starting from version 0.2.2, face detection models return **bounding box metadata** alongside embeddings:

```rust
use ahnlich_client_rs::prelude::*;
use std::collections::HashMap;

// Load image bytes
let image_bytes = std::fs::read("group_photo.jpg")?;

let inputs = vec![StoreInput {
    value: Some(store_input::Value::Image(image_bytes)),
}];

let response = client
    .convert_to_embeddings(
        "faces_store".to_string(),
        inputs,
        PreprocessAction::ModelPreprocessing,
        None,
        HashMap::new(),
    )
    .await?;

// Process each detected face
for item in response.values {
    if let Some(Variant::Multiple(multi)) = item.variant {
        println!("Detected {} faces", multi.embeddings.len());
        
        for face in multi.embeddings {
            // Access embedding
            if let Some(embedding) = &face.embedding {
                println!("Embedding size: {}", embedding.key.len());
            }
            
            // Access bounding box metadata
            if let Some(metadata) = &face.metadata {
                if let Some(bbox_x1) = metadata.value.get("bbox_x1") {
                    if let Some(metadata_value::Value::RawString(x1_str)) = &bbox_x1.value {
                        let x1: f32 = x1_str.parse().unwrap();
                        println!("Face detected at x1: {}", x1);
                    }
                }
                
                if let Some(confidence) = metadata.value.get("confidence") {
                    if let Some(metadata_value::Value::RawString(conf_str)) = &confidence.value {
                        let conf: f32 = conf_str.parse().unwrap();
                        println!("Detection confidence: {}", conf);
                    }
                }
            }
        }
    }
}
```

## Metadata Fields (Face Detection Models)

For Buffalo-L and SFace models, each detected face includes:

| Field | Type | Range | Description |
|-------|------|-------|-------------|
| `bbox_x1` | f32 | 0.0-1.0 | Normalized x-coordinate of top-left corner |
| `bbox_y1` | f32 | 0.0-1.0 | Normalized y-coordinate of top-left corner |
| `bbox_x2` | f32 | 0.0-1.0 | Normalized x-coordinate of bottom-right corner |
| `bbox_y2` | f32 | 0.0-1.0 | Normalized y-coordinate of bottom-right corner |
| `confidence` | f32 | 0.0-1.0 | Detection confidence score |

Coordinates are normalized to 0-1 range. To convert to pixel coordinates:
```rust
let pixel_x1 = bbox_x1 * image_width as f32;
let pixel_y1 = bbox_y1 * image_height as f32;
```

## Using Model Parameters

Face detection models support tuning via `model_params`:

```rust
let mut model_params = HashMap::new();
model_params.insert("confidence_threshold".to_string(), "0.9".to_string());

let response = client
    .convert_to_embeddings(
        "faces_store".to_string(),
        inputs,
        PreprocessAction::ModelPreprocessing,
        None,
        model_params, // Higher threshold = fewer but more confident detections
    )
    .await?;
```

## Response Structure

The response contains a `StoreInputToEmbeddingsList` with a vector of `SingleInputToEmbedding`:

```rust
pub struct SingleInputToEmbedding {
    pub input: Option<StoreInput>,      // Original input
    pub variant: Option<Variant>,       // OneToOne or OneToMany
}

pub enum Variant {
    Single(EmbeddingWithMetadata),                // For text/image models
    Multiple(MultipleEmbedding),                   // For face detection
}

pub struct EmbeddingWithMetadata {
    pub embedding: Option<StoreKey>,               // The embedding vector
    pub metadata: Option<StoreValue>,              // Optional metadata
}
```

## Use Cases

- **Testing models**: Quickly test how different inputs are embedded
- **Batch processing**: Generate embeddings for analysis without storage
- **Face detection**: Extract face locations and embeddings from photos
- **Quality control**: Filter low-confidence detections before storage
