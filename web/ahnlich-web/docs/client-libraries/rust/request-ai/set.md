---
title: Set
---

# Set

Inserts or updates embeddings and their associated metadata into a vector store managed by the **AI service**. This operation is central to populating a store with new data or refreshing existing entries to keep the dataset consistent and relevant for similarity search.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::ai::AiClient;
  use ahnlich_client_rs::error::AhnlichError;
  use ahnlich_types::ai::preprocess::PreprocessAction;
  use ahnlich_types::ai::query::Set;
  use ahnlich_types::keyval::{AiStoreEntry, StoreInput, StoreValue};
  use ahnlich_types::keyval::store_input::Value;
  use std::collections::HashMap;


  #[tokio::main]
  async fn main() -> Result<(), AhnlichError> {
      // Connect to AI server
      let addr = "127.0.0.1:1370";
      let client = AiClient::new(addr.to_string()).await?;


      // Prepare data for Set
      let set_params = Set {
          store: "Main0".to_string(),
          execution_provider: None,
          preprocess_action: PreprocessAction::NoPreprocessing as i32,
          inputs: vec![
              AiStoreEntry {
                  key: Some(StoreInput { value: Some(Value::RawString("Adidas Yeezy".into())) }),
                  value: Some(StoreValue { value: HashMap::new() }),
              },
              AiStoreEntry {
                  key: Some(StoreInput { value: Some(Value::RawString("Nike Air Jordans".into())) }),
                  value: Some(StoreValue { value: HashMap::new() }),
              },
          ],
      };


      // Run the set command
      let res = client.set(set_params, None).await?;
      println!("Inserted entries: {:?}", res.upsert);


      Ok(())
  }
  ```
</details>

## Parameters
* `params: Set` — The embeddings and metadata to be stored, including the store key and value. If a key already exists, this call updates the associated record.


## Returns
* `Ok(SetResult)` — Contains the outcome of the insertion or update (e.g., confirmation of success, affected keys).


* `Err(AhnlichError)` — If the operation fails due to invalid input, conflicts, or server-side errors.


## Behavior (explains the code, brief)
* Prepares the `Set` request payload inside a `tonic::Request`.

* Adds tracing context if provided.

* Sends the request via the AI client’s `set` RPC method.

* Waits for the response and extracts the typed result.

* Returns a `SetResult` indicating success or failure.
