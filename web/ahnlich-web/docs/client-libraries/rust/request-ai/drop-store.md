---
title: Drop Store
---

# Drop Store

Deletes an entire vector store from the **AI service**, including all embeddings and their associated metadata. This is a destructive operation and should be used with caution. Dropping a store is useful when the store is no longer needed, when cleaning up unused resources, or when resetting a dataset for fresh ingestion.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_types::ai::query::DropStore;
  use ahnlich_client_rs::ai::AiClient;


  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      let client = AiClient::new("http://127.0.0.1:1370".to_string()).await?;


      let drop_params = DropStore {
          store: "Deven Kicks".to_string(),
          error_if_not_exists: true,
      };


      let result = client.drop_store(drop_params, None).await?;


      println!("Deleted count: {}", result.deleted_count);


      Ok(())
  }
  ```
</details>

## Parameters
* `params: DropStore` — Specifies the store to remove.

* `tracing_id: Option<String>` — Optional trace parent ID for observability and distributed tracing.


## Returns
* `Ok(Del)` — Confirmation that the store and all its contents were successfully deleted.

* `Err(AhnlichError)` — Returned if the store does not exist, is in use, or the deletion fails.


## Behavior (explains the code, brief)
* Wraps the `DropStore` parameters in a `tonic::Request`.

* Attaches optional tracing metadata.

* Sends the request to the AI service’s RPC endpoint.

* Awaits the server response and extracts the result.

* Returns a `Del` object confirming the store’s removal.
