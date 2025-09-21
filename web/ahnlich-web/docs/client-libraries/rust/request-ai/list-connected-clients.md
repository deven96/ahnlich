---
title: List Connected Clients
---

# List Connected Clients

Retrieves a list of clients currently connected to the **AI service**. This provides visibility into active sessions, useful for monitoring, debugging, and coordinating multi-client AI workloads.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_types::db::query::CreateNonLinearAlgorithmIndex;
  use ahnlich_client_rs::AiClient; // AiClient is exposed at crate root
  use tokio;


  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      // Connect to AI client (adjust the URL if needed)
      let ai_client = AiClient::new("http://[::1]:1370".to_string()).await?;


      // Only `store` and `non_linear_indices` are valid fields
      let params = CreateNonLinearAlgorithmIndex {
          store: "Deven Kicks".to_string(),
          non_linear_indices: vec!["my_algorithm".to_string()],
      };


      // Call the RPC
      let response = ai_client
          .create_non_linear_algorithm_index(params, None)
          .await?;


      println!("Non-linear algorithm index created: {:?}", response);


      Ok(())
  }
  ```
</details>

## Returns
* `Ok(ClientList)` — A structured list of active AI client connections.

* `Err(AhnlichError)` — Returned if the request fails due to connectivity issues or service errors.


## Behavior (explains the code, brief)
* Wraps an empty `ListClients {}` request in a `tonic::Request`.

* Adds trace metadata if provided.

* Calls the AI service’s `list_clients` RPC.

* Awaits the response and extracts the list of connected clients.
