---
title: List Connected Clients
---

# List Connected Clients

Retrieves a list of clients currently connected to the **AI service**. This provides visibility into active sessions, useful for monitoring, debugging, and coordinating multi-client AI workloads.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::ai::AiClient;
  use ahnlich_client_rs::error::AhnlichError;


  #[tokio::main]
  async fn main() -> Result<(), AhnlichError> {
      // Connect to AI server
      let addr = "127.0.0.1:1370".to_string();
      let ai_client = AiClient::new(addr).await?;


      // Fetch the list of connected clients
      let clients = ai_client.list_clients(None).await?;


      // Print the clients in a readable way
      println!("Connected clients:");
      for (i, client) in clients.clients.iter().enumerate() {
          println!("{}. {:?}", i + 1, client);
      }


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
