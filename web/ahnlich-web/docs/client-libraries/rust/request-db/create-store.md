---
title: Create Store
---

# Create Store

Creates a new vector store within the Ahnlich DB service. A store is the primary container for vectors and metadata, and all vector operations must be scoped to a specific store. This request is essential for initializing logical partitions of data before inserting or querying vectors.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::db::DbClient;
  use ahnlich_types::db::query::CreateStore;


  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      // Connect to server
      let db_client = DbClient::new("127.0.0.1:1369".to_string()).await?;


      let tracing_id: Option<String> = None;


      // Define parameters for store creation
      let params = CreateStore {
          store: "Main".to_string(),
          dimension: 3,
          create_predicates: vec!["role".to_string()],
          non_linear_indices: vec![],
          error_if_exists: true,
      };


      // Call create_store
      match db_client.create_store(params, tracing_id).await {
          Ok(res) => {
              println!("Store created successfully: {:?}", res);
          }
          Err(err) => {
              eprintln!("Error creating store: {:?}", err);
          }
      }


      Ok(())
  }

  ```
</details>

## Parameters
* `params: CreateStore` – The configuration for the new store. This includes required fields such as the store name, vector dimensionality, and optional indexing options..


## Returns
* `Ok(Unit)` – Indicates that the store was successfully created.

* `Err(AhnlichError)` – Returned if the request fails. Common failure cases include:

  * A store with the same name already exists.

  * Invalid configuration parameters (e.g., mismatched dimensions).

  * Server-side or transport-level errors.




## Behavior
* This is a write operation and will allocate resources on the server.

* Once created, the store immediately becomes available for other operations such as `Set`, `Get Sim N`, or predicate-based queries.

* Tracing information, if provided, is propagated through the request for monitoring and debugging.


## Usage considerations
* Use `List Stores` after creation to verify that the store has been registered successfully. The response includes the non-linear index configurations (HNSW parameters, k-d tree) for each store.

* Stores are persistent until explicitly removed using `Drop Store`.

* Proper planning of store dimensions and indexing strategy is recommended, as these cannot be trivially changed after creation.

* When creating a store with an HNSW index, you can pass configuration parameters such as `ef_construction`, `maximum_connections`, `maximum_connections_zero`, `distance` metric, `extend_candidates`, and `keep_pruned_connections` via `HnswConfig`. If not specified, sensible defaults are used.

* A store cannot have duplicate non-linear indices of the same type. Attempting to add an HNSW index to a store that already has one will be silently ignored (idempotent).
