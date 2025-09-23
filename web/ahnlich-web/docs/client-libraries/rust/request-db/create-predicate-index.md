---
title: Create Predicate Index
---

# Create Predicate Index

Creates an index on a predicate field in a store. Predicate indexes allow efficient filtering and retrieval of vectors based on metadata conditions, improving query performance for repeated predicate lookups.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::db::DbClient;
  use ahnlich_types::db::query::CreatePredIndex;


  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      // connect to DB server
      let db_client = DbClient::new("127.0.0.1:1369".to_string()).await?;
      let tracing_id: Option<String> = None;


      // Create predicate index request
      let params = CreatePredIndex {
          store: "Main".to_string(),
          predicates: vec!["role = 'admin'".to_string()], // <-- must be Vec<String>
      };


      // Call the client
      match db_client.create_pred_index(params, tracing_id).await {
          Ok(result) => {
              println!("Created predicate index: {:?}", result);
          }
          Err(err) => {
              eprintln!("Error creating predicate index: {:?}", err);
          }
      }


      Ok(())
  }
  ```
</details>

## Parameters
* `params: CreatePredIndex` — Defines the target store and the predicate field to be indexed.


## Returns
* `CreateIndex` — Response confirming that the index has been successfully created.

* `AhnlichError` — Returned if the store does not exist, the field is invalid, or index creation fails.


## Behavior
* Wraps the provided `CreatePredIndex` parameters into a gRPC request.

* Adds optional tracing metadata to the request for debugging or monitoring.

* Invokes the `create_pred_index` RPC on the Ahnlich server via the cloned gRPC client.

* Awaits the server’s response and extracts the `CreateIndex` result using `.into_inner()`.
