---
title: Create Store
---

# Create Store

Creates a new vector store within the AI service. A store acts as a container for embeddings and metadata, enabling structured organization of data for similarity search and retrieval tasks. This is typically the first step before inserting embeddings or performing queries against a specific dataset.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_types::ai::query::{CreateStore, DropStore};
  use ahnlich_client_rs::ai::AiClient;
  use ahnlich_types::ai::models::AiModel;


  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      let client = AiClient::new("http://127.0.0.1:1370".to_string()).await?;


      // Create store first
      let create_params = CreateStore {
          store: "Deven Kicks".to_string(),
          index_model: AiModel::AllMiniLmL6V2 as i32,
          query_model: AiModel::AllMiniLmL6V2 as i32,
          predicates: vec![],
          non_linear_indices: vec![],
          error_if_exists: false,
          store_original: true,
      };
      client.create_store(create_params, None).await?;


      // Now drop it
      let drop_params = DropStore {
          store: "MyStore".to_string(),
          error_if_not_exists: true,
      };


      let result = client.drop_store(drop_params, None).await?;


      println!("Deleted count: {}", result.deleted_count);


      Ok(())
  }
  ```
</details>

## Parameters
* `params: CreateStore` — Input parameters defining the new store (e.g., store name, configuration options).


* `tracing_id: Option<String>` — Optional trace parent for distributed tracing, included in the request if provided.


## Returns
* `Ok(Unit)` — A confirmation response indicating that the store was successfully created.

* `Err(AhnlichError)` — If creation fails due to invalid parameters, a name conflict, or service errors.


## Behavior (explains the code, brief)
* Wraps the `CreateStore` parameters in a `tonic::Request`.

* Attaches the tracing ID if provided for observability.

* Invokes the `create_store` RPC on the AI client.

* Awaits the server’s response and extracts the result.

* Returns `Unit` to signal successful completion.
