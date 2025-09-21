---
title: Delete Key
---

# Delete Key

Deletes a specific embedding and its associated metadata from a vector store in the **AI service**. This operation is useful for removing obsolete or incorrect embeddings, ensuring that similarity searches and AI queries return only relevant results.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::ai::AiClient;
  use ahnlich_client_rs::error::AhnlichError;
  use ahnlich_types::ai::query::DelKey;
  use ahnlich_types::ai::server::Del;
  use ahnlich_types::keyval::StoreInput;
  use ahnlich_types::keyval::store_input::Value;


  #[tokio::main]
  async fn main() -> Result<(), AhnlichError> {
      // Connect to server
      let ai_client = AiClient::new("http://127.0.0.1:1370".to_string())
          .await
          .expect(" Failed to connect AI client");


      // Define key to delete
      let params = DelKey {
          store: "Deven Kicks".to_string(),
          keys: vec![StoreInput {
              value: Some(Value::RawString("Nike Air Jordans".to_string())),
          }],
      };


      // Call delete
      let response: Del = ai_client.del_key(params, None).await?;
      println!(" Deleted key result: {:?}", response);


      Ok(())
  }
  ```
</details>

## Parameters
* `params: DelKey` — Specifies the store and the unique key of the embedding to remove.

* `tracing_id: Option<String>` — Optional trace parent ID for observability and distributed tracing.


## Returns
* `Ok(Del)` — Confirmation that the embedding and metadata were successfully deleted.

* `Err(AhnlichError)` — Returned if the key does not exist, the store is unavailable, or the deletion fails.


## Behavior (explains the code, brief)
* Wraps the `DelKey` parameters in a `tonic::Request`.

* Attaches tracing metadata if provided.

* Sends the deletion request to the AI service via RPC.

* Awaits the response and extracts the result.

* Returns a `Del` object indicating successful deletion.
