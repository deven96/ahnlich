---
title: Get Store
---

# Get Store

Retrieves detailed information about a specific AI store by name, including the configured models and optional underlying DB store information.

## Source Code Example

<details>
  <summary>Click to expand</summary>

```rust
use ahnlich_client_rs::ai::AiClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ai_client = AiClient::new("127.0.0.1:1370".to_string()).await?;

    let tracing_id: Option<String> = None;

    let store_info = ai_client
        .get_store("ai_store".to_string(), tracing_id)
        .await?;

    println!("Store name: {}", store_info.name);
    println!("Query model: {:?}", store_info.query_model);
    println!("Index model: {:?}", store_info.index_model);
    println!("Embedding size: {}", store_info.embedding_size);
    println!("Dimension: {}", store_info.dimension);
    println!("Predicate indices: {:?}", store_info.predicate_indices);

    if let Some(db_info) = &store_info.db_info {
        println!("DB store size: {} bytes", db_info.size_in_bytes);
    }

    Ok(())
}
```
</details>

## Parameters

* `store: String` — The name of the AI store to retrieve.

* `tracing_id: Option<String>` — Optional trace context for observability.

## Returns

* `AiStoreInfo` — Detailed information about the AI store.

* `AhnlichError` — If the store does not exist or the request fails.

## AiStoreInfo Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Store name |
| `query_model` | `AiModel` | AI model used for query embeddings |
| `index_model` | `AiModel` | AI model used for index embeddings |
| `embedding_size` | `u64` | Number of stored embeddings |
| `dimension` | `u32` | Vector dimension (determined by model) |
| `predicate_indices` | `Vec<String>` | List of indexed predicate keys |
| `db_info` | `Option<StoreInfo>` | Underlying DB store info (when connected) |

## Behavior

* Sends a request to retrieve AI store metadata by name.

* Returns an error if the store does not exist.

* The `db_info` field is present when the AI proxy is connected to a DB instance.

## Notes

- Use `list_stores` to get information about all AI stores
- The model fields indicate which embedding models are used for indexing and querying
