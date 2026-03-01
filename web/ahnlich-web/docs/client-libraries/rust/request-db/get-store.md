---
title: Get Store
---

# Get Store

Retrieves detailed information about a specific store by name. Returns metadata including dimensions, size, and configured indices.

## Source Code Example

<details>
  <summary>Click to expand</summary>

```rust
use ahnlich_client_rs::db::DbClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let db_client = DbClient::new("127.0.0.1:1369".to_string()).await?;

    let tracing_id: Option<String> = None;

    let store_info = db_client
        .get_store("my_store".to_string(), tracing_id)
        .await?;

    println!("Store name: {}", store_info.name);
    println!("Number of entries: {}", store_info.len);
    println!("Size in bytes: {}", store_info.size_in_bytes);
    println!("Dimension: {}", store_info.dimension);
    println!("Predicate indices: {:?}", store_info.predicate_indices);
    println!("Non-linear indices: {:?}", store_info.non_linear_indices);

    Ok(())
}
```
</details>

## Parameters

* `store: String` — The name of the store to retrieve.

* `tracing_id: Option<String>` — Optional trace context for observability.

## Returns

* `StoreInfo` — Detailed information about the store.

* `AhnlichError` — If the store does not exist or the request fails.

## StoreInfo Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | `String` | Store name |
| `len` | `u64` | Number of entries in the store |
| `size_in_bytes` | `u64` | Total size of the store in bytes |
| `dimension` | `u32` | Vector dimension |
| `predicate_indices` | `Vec<String>` | List of indexed predicate keys |
| `non_linear_indices` | `Vec<NonLinearIndex>` | List of non-linear algorithm indices |

## Behavior

* Sends a request to retrieve store metadata by name.

* Returns an error if the store does not exist.

* Useful for inspecting store configuration before operations.

## Notes

- Use `list_stores` to get information about all stores
- The `size_in_bytes` field is useful for monitoring memory usage
