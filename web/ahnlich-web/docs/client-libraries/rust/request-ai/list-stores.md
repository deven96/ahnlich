---
title: List Store
---

# List Stores

Retrieves a list of all vector stores currently managed by the **AI service**. Each store represents a logical container for embeddings and their associated metadata. This operation is useful for exploring available stores before performing read or write operations.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::ai::AiClient;
  use ahnlich_client_rs::error::AhnlichError;


  #[tokio::main]
  async fn main() -> Result<(), AhnlichError> {
      let addr = "127.0.0.1:1370";
      let client = AiClient::new(addr.to_string()).await?;


      let stores = client.list_stores(None).await?;
      println!("Stores: {:?}", stores);


      Ok(())
  }
  ```
</details>

## Returns
* `Ok(StoreList)` — A structured list of available vector stores managed by the AI service.

* `Err(AhnlichError)` — If the request fails due to connectivity issues, authorization errors, or service unavailability.


## Behavior (explains the code, brief)
* Creates a `tonic::Request` wrapping an empty `ListStores {}` message.

* Adds a tracing ID if provided for observability.

* Calls the remote `list_stores` RPC using the AI client.

* Awaits the result and unwraps the server’s response.

* Returns the `StoreList` object containing store metadata.


