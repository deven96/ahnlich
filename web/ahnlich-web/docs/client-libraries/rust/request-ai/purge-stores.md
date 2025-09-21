---
title: Purge Stores
---

# Purge Stores

Deletes **all vector stores** managed by the **AI client**, including all embeddings and associated metadata. This is a destructive operation that resets the AI service state, typically used during testing, cleanup, or when starting fresh with new datasets.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::ai::{AiClient, AiPipeline};
  use ahnlich_types::ai::models::AiModel;
  use ahnlich_types::ai::query::CreateStore;
  use ahnlich_types::ai::pipeline::AiResponsePipeline;
  use tokio::time::Duration;


  #[tokio::main]
  async fn main() {
      // Initialize AI client (replace with your server address)
      let ai_client = AiClient::new("http://127.0.0.1:1370".to_string())
          .await
          .expect("Could not connect to AI client");


      // Create a new pipeline
      let mut pipeline = ai_client.pipeline(None);


      // Example: create a test store
      let store = CreateStore {
          store: "TestStore".to_string(),
          index_model: AiModel::AllMiniLmL6V2 as i32,
          query_model: AiModel::AllMiniLmL6V2 as i32,
          predicates: vec![],
          non_linear_indices: vec![],
          error_if_exists: true,
          store_original: true,
      };


      pipeline.create_store(store);


      // You can add more pipeline actions here
      // Example: purge all stores
      pipeline.purge_stores();


      // Execute pipeline
      let res: AiResponsePipeline = pipeline.exec().await.expect("Pipeline execution failed");


      println!("Pipeline result: {res:#?}");
  }
  ```
</details>

## Returns
* `Ok(Del)` — Confirmation that all stores and their contents were successfully deleted.

* `Err(AhnlichError)` — Returned if the operation fails due to service errors or connectivity issues.


## Behavior (explains the code, brief)
* Wraps an empty `PurgeStores {}` request in a `tonic::Request`.

* Attaches optional tracing metadata.

* Sends the request to the AI service via RPC.

* Awaits the server response and extracts the result.

* Returns a `Del` object indicating successful deletion of all stores.