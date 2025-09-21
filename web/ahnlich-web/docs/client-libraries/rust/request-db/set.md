---
title: Set
---

# Set

The `Set` request inserts or updates a vector in a given store. Each vector is stored alongside optional metadata and a unique key. If a key already exists in the store, calling `Set` with the same key will overwrite the existing vector and metadata.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::db::DbClient;
  use ahnlich_types::{
      db::query::Set,
      keyval::{DbStoreEntry, StoreKey, StoreValue},
      metadata::{MetadataValue, metadata_value::Value},
  };
  use std::collections::HashMap;


  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      // Connect to DB server
      let db_client = DbClient::new("127.0.0.1:1369".to_string()).await?;


      let tracing_id: Option<String> = None;


      // Construct inputs for the "set"
      let inputs = vec![DbStoreEntry {
          key: Some(StoreKey {
              key: vec![0.5, 0.2, 0.9], // must match store dimension
          }),
          value: Some(StoreValue {
              value: HashMap::from_iter([(
                  "role".into(),
                  MetadataValue {
                      value: Some(Value::RawString("admin".into())),
                  },
              )]),
          }),
      }];


      let params = Set {
          store: "Main".to_string(), // store must already exist
          inputs,
      };


      // Call set
      match db_client.set(params, tracing_id).await {
          Ok(result) => {
              println!("Set operation result: {:?}", result);
          }
          Err(err) => {
              eprintln!("Error inserting vector: {:?}", err);
          }
      }


      Ok(())
  }
  ```
</details>

## Parameters
* `params: Set` – Defines the data to be stored. This includes:

  * `store` – The name of the target store.

  * `key` – A unique identifier for the vector.

  * `vector` – The vector data (as `Vec<f32>`).

  * `metadata` – Optional key–value pairs to annotate the vector (e.g., labels, categories).


* `tracing_id: Option<String>` – An optional tracing identifier for distributed observability.


## Returns
* `Ok(SetResult)` – Contains information about the completed operation, such as success status and assigned identifiers.

* `Err(AhnlichError)` – Returned if the operation fails. Common error cases include:

  * Target store does not exist.

  * Provided vector does not match the dimensionality of the store.

  * Invalid key or malformed metadata.

  * Transport or server-side errors.


## Behavior
* `Set` is an upsert operation: it will insert if the key does not exist, or update if it already exists.

* Vectors written with `Set` are immediately available for similarity searches (e.g., `Get Sim N`) or retrieval by key.

* Metadata can be leveraged in predicate-based queries if a predicate index is defined.

* Ensure consistent key usage to avoid unintended overwrites.
