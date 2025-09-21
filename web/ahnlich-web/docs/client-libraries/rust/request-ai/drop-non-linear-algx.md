---
title: Drop Non-Linear Algorithm Index
---

# Drop Non-Linear Algorithm Index

Removes a **non-linear algorithm index** from a vector store in the **AI service**. This operation is useful when the index is no longer needed, when changing algorithm parameters, or when rebuilding the index for updated embedding datasets. Removing unused indexes helps maintain storage efficiency and avoids unnecessary overhead during similarity searches.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::ai::AiClient;
  use ahnlich_types::ai::query::DropNonLinearAlgorithmIndex;
  use ahnlich_types::algorithm::nonlinear::NonLinearAlgorithm;
  use tokio;


  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      // Connect to AI server
      let client = AiClient::new("http://127.0.0.1:1370".to_string()).await?;


      // Set parameters for dropping the non-linear index
      let params = DropNonLinearAlgorithmIndex {
          store: "MyStore".to_string(),
          non_linear_indices: vec![NonLinearAlgorithm::KdTree as i32],
          error_if_not_exists: true, // do not error if the index doesn't exist
      };


      // Execute drop request
      let result = client
          .drop_non_linear_algorithm_index(params, None)
          .await?;
    
      println!("Dropped non-linear indices: {:?}", result);


      Ok(())
  }
  ```
</details>

## Parameters
* `params: DropNonLinearAlgorithmIndex` — Specifies the store and the non-linear index to remove.


## Returns
* `Ok(Del)` — Confirmation that the non-linear algorithm index was successfully deleted.

* `Err(AhnlichError)` — Returned if the index does not exist, the store is unavailable, or the operation fails.


## Behavior (explains the code, brief)
* Wraps the `DropNonLinearAlgorithmIndex` parameters in a `tonic::Request`.

* Attaches optional tracing metadata.

* Sends the request to the AI service via RPC.

* Awaits the response and extracts the result.

* Returns a `Del` object indicating successful deletion of the index.


