---
title: Create Non-Linear Algorithm Index
---

# Create Non-Linear Algorithm Index

## Schema

This request accepts an optional `schema` field. When it is omitted, the server uses the `public` schema. Set `schema` to target a store in another schema.

Creates a non-linear algorithm index on a store to optimize vector similarity searches beyond basic linear methods. Non-linear indexes (HNSW) improve query performance and scalability when working with large vector datasets.

Each index type is specified using a `NonLinearIndex` message with a `HnswConfig`.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::db::DbClient;
  use ahnlich_types::db::query::CreateNonLinearAlgorithmIndex;
  use ahnlich_types::algorithm::nonlinear::{NonLinearIndex, non_linear_index, HnswConfig};


  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      let db_client = DbClient::new("127.0.0.1:1369".to_string()).await?;

      // Create an HNSW index (with default config) on the "Main" store
      let params = CreateNonLinearAlgorithmIndex {
          store: "Main".to_string(),
          schema: Some("analytics".to_string()),
          non_linear_indices: vec![
              NonLinearIndex {
                  index: Some(non_linear_index::Index::Hnsw(HnswConfig::default())),
              },
          ],
      };

      let result = db_client.create_non_linear_algorithm_index(params, None).await?;
      println!("Created non-linear indices: {:?}", result);

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


