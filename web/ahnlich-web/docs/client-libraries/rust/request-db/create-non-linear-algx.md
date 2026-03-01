---
title: Create Non-Linear Algorithm Index
---

# Create Non-Linear Algorithm Index

Creates a non-linear algorithm index on a store to optimize vector similarity searches beyond basic linear methods. Non-linear indexes (KDTree, HNSW) improve query performance and scalability when working with large vector datasets.

Each index type is specified using a `NonLinearIndex` message with either a `KdTreeConfig` or `HnswConfig`.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::error::AhnlichError;
  use ahnlich_types::{
      db::query::CreateNonLinearAlgorithmIndex,
      algorithm::nonlinear::{NonLinearIndex, non_linear_index, KdTreeConfig, HnswConfig},
      services::db_service::db_service_client::DbServiceClient,
  };
  use tonic::transport::Channel;


  #[tokio::main]
  async fn main() -> Result<(), AhnlichError> {
      // Connect to your DB server
      let client = DbServiceClient::<Channel>::connect("http://127.0.0.1:1369").await?;

      // Create a KDTree index on the "Main" store
      let params = CreateNonLinearAlgorithmIndex {
          store: "Main".to_string(),
          non_linear_indices: vec![
              NonLinearIndex {
                  index: Some(non_linear_index::Index::Kdtree(KdTreeConfig {})),
              },
          ],
      };

      let response = client
          .clone()
          .create_non_linear_algorithm_index(params)
          .await?
          .into_inner();

      // Or create an HNSW index (with default config)
      let params = CreateNonLinearAlgorithmIndex {
          store: "Main".to_string(),
          non_linear_indices: vec![
              NonLinearIndex {
                  index: Some(non_linear_index::Index::Hnsw(HnswConfig::default())),
              },
          ],
      };

      let response = client
          .clone()
          .create_non_linear_algorithm_index(params)
          .await?
          .into_inner();

      println!("Non-linear algorithm index created: {:?}", response);

      Ok(())
  }
  ```
</details>

## Parameters
* `params: CreateNonLinearAlgorithmIndex` — Defines the target store and configuration for the non-linear index.

* `tracing_id: Option<String>` — Optional trace context for observability.


## Returns
* `CreateIndex` — Confirmation and details of the newly created index.

* `AhnlichError` — If the request fails due to invalid parameters or server errors.


## Behavior
* Builds a request with the provided `params`.

* Attaches optional tracing metadata for distributed tracing.

* Sends the request to the DB service to create the index.


