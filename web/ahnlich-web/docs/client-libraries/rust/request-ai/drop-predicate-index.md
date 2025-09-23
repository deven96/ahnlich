---
title: Drop Predicate Index
---

# Drop Predicate Index

Removes a **predicate index** used by the AI service to optimize filtered embedding queries. This operation is useful when certain metadata-based filters are no longer needed for semantic search, or when the index must be rebuilt due to changes in input fields.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::ai::AiClient;
  use ahnlich_client_rs::error::AhnlichError;
  use ahnlich_types::ai::query::DropPredIndex;
  use ahnlich_types::ai::server::Del;


  #[tokio::main]
  async fn main() -> Result<(), AhnlichError> {
      // Connect to the AI service
      let ai_client = AiClient::new("http://127.0.0.1:1370".to_string())
          .await
          .expect("Failed to connect AI client");


      // Define which store and which predicate index to drop
      let params = DropPredIndex {
          store: "Deven Kicks".to_string(),
          predicates: vec!["Brand".to_string()],
          error_if_not_exists: true, // 👈 required field, prevents silent no-op
      };


      // Call the API
      let response: Del = ai_client.drop_pred_index(params, None).await?;


      println!(" Dropped predicate index result: {:?}", response);


      Ok(())
  }
  ```
</details>

## Parameters
* `params: DropPredIndex` — Specifies the store and the predicate index to remove.

* `tracing_id: Option<String>` — Optional trace parent ID for observability and distributed tracing.


## Returns
* `Ok(Del)` — Confirmation that the predicate index was successfully removed.

* `Err(AhnlichError)` — Returned if the index does not exist, the store is unavailable, or the request fails.


## Behavior (explains the code, brief)
* Wraps the `DropPredIndex` parameters in a `tonic::Request`.

* Attaches optional tracing metadata.

* Sends the request to the AI service.

* Awaits the response and extracts the result.

* Returns a `Del` object confirming the deletion.