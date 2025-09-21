---
title: Create Predicate Index
---

# Create Predicate Index

Creates a predicate index in the AI service to optimize filtered embedding queries. Speeds up retrieval of embeddings based on metadata constraints, improving the performance of `get_pred` operations.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::ai::AiClient;
  use ahnlich_client_rs::error::AhnlichError;
  use ahnlich_types::ai::query::CreatePredIndex;
  use ahnlich_types::ai::server::CreateIndex;


  #[tokio::main]
  async fn main() -> Result<(), AhnlichError> {
      // Connect to the AI service
      let ai_client = AiClient::new("http://127.0.0.1:1370".to_string())
          .await
          .expect("Failed to connect AI client");


      // Define which store and which predicates to index
      let params = CreatePredIndex {
          store: "Deven Kicks".to_string(),
          predicates: vec!["Brand".to_string(), "Vintage".to_string()],
      };


      // Call the API
      let response: CreateIndex = ai_client.create_pred_index(params, None).await?;


      println!(" Created predicate indexes: {:?}", response);


      Ok(())
  }
  ```
</details>

## Parameters
* `params: CreatePredIndex` — Specifies the store and the metadata fields to index for predicate-based queries.

* `tracing_id: Option<String>` — Optional trace parent ID for observability and distributed tracing.


## Returns
* `Ok(CreateIndex)` — Confirmation that the predicate index was successfully created, along with index details.

* `Err(AhnlichError)` — Returned if index creation fails due to invalid parameters, store issues, or server errors.


## Behavior (explains the code, brief)
* Wraps the `CreatePredIndex` input in a `tonic::Request`.

* Adds optional tracing metadata.

* Calls the AI/DB service’s `create_pred_index` RPC endpoint.

* Waits for the response and extracts the index creation result.

* Returns the newly created index information.
