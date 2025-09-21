---
title: New
---

# New

Initializes a new **AI client** instance that can communicate with the Ahnlich **AI service**. This method sets up the gRPC connection and prepares the client for performing operations such as embedding insertions, similarity searches, and pipelines.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::ai::AiClient;
  use ahnlich_client_rs::error::AhnlichError;
  use tokio;


  #[tokio::main]
  async fn main() -> Result<(), AhnlichError> {
      let addr = "127.0.0.1:1370".to_string();


      match AiClient::new(addr).await {
          Ok(_client) => {
              println!("AiClient successfully created!");
              Ok(())
          }
          Err(e) => {
              eprintln!("Failed to create AiClient: {:?}", e);
              Err(e)
          }
      }
  }
  ```
</details>

## Parameters
* `addr: String` — The address of the AI service (e.g., `"127.0.0.1:1369"`). If the scheme (`http://` or `https://`) is not provided, it defaults to `http://`.


## Returns
* `Ok(AiClient)` — A fully initialized AI client ready to perform requests.

* `Err(AhnlichError)` — Returned if the connection fails or the provided address is invalid.


## Behavior (explains the code, brief)
* Ensures the address includes a valid HTTP/HTTPS scheme.

* Creates a gRPC `Channel` from the given address.

* Connects to the AI service using `AiServiceClient::connect`.

* Wraps the client in an `AiClient` struct and returns it.
