---
title: Drop Store
---

# Drop Store

Deletes an entire store from the database, including all vectors, keys, and associated metadata. This is a destructive operation and cannot be reversed—once a store is dropped, all of its contents are permanently removed.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_types::db::query::DropStore;
  use ahnlich_client_rs::db::DbClient;


  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      // Connect to the DB server
      let client = DbClient::new("http://127.0.0.1:1369".to_string()).await?;


      // Prepare drop store parameters
      let drop_params = DropStore {
          store: "MyStore".to_string(),
          error_if_not_exists: true, // Required field
      };


      // Execute the drop store request
      let result = client.drop_store(drop_params, None).await?;


      println!("Deleted count: {}", result.deleted_count);


      Ok(())
  }
  ```
</details>

## Parameters
* `params: DropStore` — Identifies the store to be deleted.

* `tracing_id: Option<String>` — Optional trace context for observability.


## Returns
* `Del` — Confirmation that the store was successfully dropped.

* `AhnlichError` — If the store does not exist or the operation fails.


## Behavior
* Builds a gRPC request with the `DropStore` parameters.

* Attaches tracing metadata if provided.

* Calls the DB client to drop the specified store and remove all its contents.
