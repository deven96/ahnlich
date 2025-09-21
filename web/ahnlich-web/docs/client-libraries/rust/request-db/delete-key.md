---
title: Delete Key
---

# Delete Key

Removes a single key and its associated vector and metadata from a store. This operation permanently deletes the entry, ensuring it is no longer retrievable in similarity searches or direct lookups. Use this to manage lifecycle of individual records without affecting the rest of the store.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::db::DbClient;
  use ahnlich_types::db::query::DelKey;
  use ahnlich_types::keyval::StoreKey;
  use tokio;


  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      let addr = "http://127.0.0.1:1369"; // adjust if your server runs elsewhere
      let client = DbClient::new(addr.to_string()).await?;


      // Delete a specific key from store "Main" (dimension must match)
      let del_key_params = DelKey {
          store: "Main".to_string(),
          keys: vec![StoreKey {
              key: vec![1.0, 1.1, 1.2], // ✅ matches dimension=3
          }],
      };


      match client.del_key(del_key_params, None).await {
          Ok(result) => println!("Deleted count: {:?}", result.deleted_count),
          Err(e) => eprintln!("Error: {:?}", e),
      }


      Ok(())
  }
  ```
</details>

## Parameters
* `params: DelKey` — Identifies the store and the specific key to delete.

* `tracing_id: Option<String>` — Optional trace context for observability.


## Returns
* `Del` — Confirmation that the key was successfully removed.

* `AhnlichError` — If the key or store does not exist.


## Behavior
* Builds a gRPC request targeting the key specified.

* Adds optional trace metadata for distributed tracing.

* Executes the `del_key` RPC, removing the vector and metadata tied to the key.