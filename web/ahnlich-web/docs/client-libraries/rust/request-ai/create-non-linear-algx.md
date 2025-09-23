---
title: Create Non-Linear Algorithm Index
---

# Create Non-Linear Algorithm Index

Creates a **non-linear algorithm index** on a vector store within the **AI service** to optimize similarity search performance. These indexes accelerate nearest-neighbor and semantic searches over large embedding datasets, making retrieval faster and more efficient.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::ai::AiClient;
  use ahnlich_types::ai::query::CreateNonLinearAlgorithmIndex;
  use ahnlich_types::algorithm::nonlinear::NonLinearAlgorithm;
  use tokio;


  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      let client = AiClient::new("http://127.0.0.1:1370".to_string()).await?;


      let params = CreateNonLinearAlgorithmIndex {
          store: "MyStore".to_string(),
          non_linear_indices: vec![NonLinearAlgorithm::KdTree as i32],
      };


      let result = client.create_non_linear_algorithm_index(params, None).await?;
      println!("Created non-linear indices: {:?}", result);


      Ok(())
  }
  ```
</details>

## Parameters
* `params: CreateNonLinearAlgorithmIndex` — Specifies the target store and algorithm parameters for building the non-linear index.


## Returns
* `Ok(CreateIndex)` — Confirmation that the non-linear algorithm index was successfully created, including index details.

* `Err(AhnlichError)` — Returned if index creation fails due to invalid parameters, store issues, or server errors.


## Behavior (explains the code, brief)
* Wraps the `CreateNonLinearAlgorithmIndex` input in a `tonic::Request`.

* Attaches tracing metadata if provided.

* Calls the AI service’s `create_non_linear_algorithm_index` RPC endpoint.

* Awaits the response and extracts the result.

* Returns a `CreateIndex` object with details of the created index.
