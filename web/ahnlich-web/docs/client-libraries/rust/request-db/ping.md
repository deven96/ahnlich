---
title: Ping
---

# Ping

Checks connectivity with the Ahnlich service and verifies that the server can accept requests over the current gRPC channel. Useful for health checks, readiness probes, or establishing a baseline before issuing data operations.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::db::DbClient;


  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      // Connect to your running ahnlich-db instance
      let db_client = DbClient::new("127.0.0.1:1369".to_string()).await?;


      // Optional tracing ID (can be None if you don’t use tracing)
      let tracing_id: Option<String> = None;


      // Call ping and print the response
      let res = db_client.ping(tracing_id).await?;
      println!("Ping response: {:?}", res);


      Ok(())
  }

  ```
</details>

## Parameters

* `tracing_id: Option<String>` – Optional tracing context to propagate to the server.

  * Pass `Some(String)` to enable distributed tracing for this call.

  * Pass `None` to omit tracing metadata.


## Returns
* `Ok(Pong)` – A server “pong” response indicating the service is reachable.


* `Err(AhnlichError)` – The request could not be completed (e.g., transport error, server error).


## Behavior
* Executes a lightweight RPC with no side effects.

* Safe to call at startup, during liveness/readiness checks, or before building pipelines.

* Works identically on both **DB** and **AI** clients.