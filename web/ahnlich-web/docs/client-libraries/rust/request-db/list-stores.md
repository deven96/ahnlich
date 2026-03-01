---
title: List Stores
---

# List Stores

Returns the list of vector stores registered in the connected Ahnlich DB service. This request is typically used to discover available stores before performing store-scoped operations such as creating, dropping, or inserting vectors.

## Source Code Example
<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::db::DbClient;


  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      // Connect to your running ahnlich-db instance
      let db_client = DbClient::new("127.0.0.1:1369".to_string()).await?;


      let tracing_id: Option<String> = None;


      // Call list_stores and print the result
      let stores = db_client.list_stores(tracing_id).await?;
      println!("Stores: {:?}", stores);


      Ok(())
  }
  ```
</details>

## Parameters
* `tracing_id: Option<String>` – Optional tracing context propagated with the request.


## Returns
* `Ok(StoreList)` – Contains metadata for each store available on the server. Each `StoreInfo` includes:
  * `name` – The store identifier.
  * `len` – Number of entries in the store.
  * `size_in_bytes` – Total memory footprint of the store.
  * `non_linear_indices` – List of non-linear index configurations (HNSW with full config parameters, or k-d tree) active on the store. Empty if no non-linear indices are configured.

* `Err(AhnlichError)` – Returned when the request cannot be completed (e.g., transport or server error).


## Behavior
* Executes a read-only RPC with no side effects.

* Responses are deterministic: the server returns all currently known stores.

* If no stores exist, the response will contain an empty list.

* For stores with HNSW indices, the returned configuration includes: distance metric, ef_construction, maximum_connections, maximum_connections_zero, extend_candidates, and keep_pruned_connections.
