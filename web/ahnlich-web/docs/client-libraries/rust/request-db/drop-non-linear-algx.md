---
title: Drop Non-Linear Algorithm Index
---

# Drop Non-Linear Algorithm Index

Removes an existing non-linear algorithm index from a store. This operation is useful when an index is no longer needed, when switching to a different indexing strategy, or during cleanup of store resources. Dropping the index reverts the store back to standard linear search behavior unless another index exists.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_types::db::query::DropNonLinearAlgorithmIndex;
  use ahnlich_types::db::server::Del;
  use ahnlich_client_rs::db::DbClient;


  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      let addr = "http://127.0.0.1:1369".to_string();
      let db_client = DbClient::new(addr).await?;


      let params = DropNonLinearAlgorithmIndex {
          store: "Main".to_string(),
          non_linear_indices: vec![0, 1],
          error_if_not_exists: false,
      };


      match db_client
          .drop_non_linear_algorithm_index(params, None)
          .await
      {
          Ok(Del { deleted_count }) => {
              println!("Successfully dropped {} non-linear index(es).", deleted_count);
          }
          Err(e) => {
              eprintln!("Error dropping non-linear index: {:?}", e);
          }
      }


      Ok(())
  }
  ```
</details>

## Parameters
* `params: DropNonLinearAlgorithmIndex` — Specifies the target store and index to be dropped.

* `tracing_id: Option<String>` — Optional trace identifier for observability.


## Returns
* `Del` — Confirmation that the index has been dropped.

* `AhnlichError` — If the operation fails due to invalid parameters or missing index.


## Behavior
* Builds a request with the given `params`.

* Adds tracing information when provided.

* Sends the request to the DB service to drop the index.

* Returns confirmation once the index has been removed.

* Cleaning up unused indexes.

* Switching from non-linear search back to linear search.

* Replacing an old index with a newly configured one.
