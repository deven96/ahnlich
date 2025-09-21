---
title: List Connected Clients
---

# List Connected Clients

Retrieves a list of all clients currently connected to the database service. This is useful for monitoring active sessions, debugging connectivity issues, and gaining visibility into which applications are using the DB.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::db::DbClient;
  use ahnlich_client_rs::error::AhnlichError;


  #[tokio::main]
  async fn main() -> Result<(), AhnlichError> {
      // Set the DB server address
      let addr = "127.0.0.1:1369".to_string();


      // Initialize the DB client
      let db_client = DbClient::new(addr).await?;


      // Fetch the list of connected clients
      let clients = db_client.list_clients(None).await?;


      // Print the clients in a readable way
      println!("Connected clients:");
      for (i, client) in clients.clients.iter().enumerate() {
          println!("{}. {:?}", i + 1, client);
      }


      Ok(())
  }
  ```
</details>

## Parameters
* `tracing_id: Option<String>` — Optional trace context for observability.


## Returns
* `ClientList` — A structured list of connected clients with their identifiers and metadata.

* `AhnlichError` — If the request fails due to communication or server-side errors.


## Behavior
* Constructs a `ListClients` request.

* Adds optional tracing information.

* Queries the DB service for all currently connected clients and returns the result.
