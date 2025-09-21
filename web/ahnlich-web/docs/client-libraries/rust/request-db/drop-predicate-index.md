---
title: Drop Predicate Index
---

# Drop Predicate Index

Removes an existing predicate index from a store. This operation cleans up indexes that are no longer needed for query acceleration.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::db::DbClient;
  use ahnlich_types::db::query::DropPredIndex;
  use tokio;


  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      let addr = "http://127.0.0.1:1369";
      let client = DbClient::new(addr.to_string()).await?;


      // Drop the "role" predicate index from store "Main"
      let drop_index_params = DropPredIndex {
          store: "Main".to_string(),
          predicates: vec!["role".to_string()],
          error_if_not_exists: true, // fail if it doesn't exist
      };


      match client.drop_pred_index(drop_index_params, None).await {
          Ok(result) => println!("Dropped predicate index: {:?}", result),
          Err(e) => eprintln!("Error: {:?}", e),
      }


      Ok(())
  }
  ```
</details>

## Parameters
* `params: DropPredIndex` — Identifies the store and the predicate index to remove.


## Returns
* `Del` — Confirmation of index removal.

* `AhnlichError` — If the store or index does not exist.


## Behavior
* Builds a gRPC request with the target index details.

* Attaches optional trace metadata.

* Executes the `drop_pred_index` RPC to delete the index.