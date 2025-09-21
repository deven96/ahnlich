---
title: Get Key
---

# Get Key

Fetches a record from a vector store by its unique key. This provides a deterministic lookup of a specific embedding and its metadata, useful for retrieving known vectors or verifying insertion results.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::ai::AiClient;
  use ahnlich_client_rs::error::AhnlichError;
  use ahnlich_types::ai::query::GetKey;
  use ahnlich_types::keyval::store_input::Value;
  use ahnlich_types::keyval::StoreInput;


  #[tokio::main]
  async fn main() -> Result<(), AhnlichError> {
      let addr = "127.0.0.1:1370";
      let client = AiClient::new(addr.to_string()).await?;


      let keys = vec![
          StoreInput { value: Some(Value::RawString("Adidas Yeezy".to_string())) },
          StoreInput { value: Some(Value::RawString("Nike Air Jordans".to_string())) },
      ];


      let params = GetKey {
          store: "Main0".to_string(),
          keys, // directly pass the Vec
      };


      let result = client.get_key(params, None).await?;


      for entry in result.entries {
          if let Some(k) = entry.key {
              println!("Key retrieved: {:?}", k.value);
          }
      }


      Ok(())
  }
  ```
</details>

## Parameters
* `params: GetKey` — The input containing the store name and the unique key of the record to retrieve.

* `tracing_id: Option<String>` — Optional trace parent ID for distributed observability and tracing.


## Returns
* `Ok(Get)` — Contains the retrieved record, including its vector embedding and associated metadata.

* `Err(AhnlichError)` — Returned if the key does not exist, the store is unavailable, or the request fails.


## Behavior (explains the code, brief)
* Wraps the `GetKey` request parameters in a `tonic::Request`.

* Attaches trace propagation metadata if provided.

* Forwards the request to the AI service’s `get_key` RPC endpoint.

* Awaits the response and unwraps the result.

* Returns the `Get` object with the stored vector and metadata.
