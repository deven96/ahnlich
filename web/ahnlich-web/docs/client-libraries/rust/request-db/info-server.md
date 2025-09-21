---
title: Info Server
---

# Info Server
Retrieves server information for the connected Ahnlich service over the current gRPC channel. Use this to confirm service identity and inspect basic runtime details before issuing data operations or when troubleshooting.

## Source Code Example
<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::db::DbClient;


  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      // Connect to your ahnlich-db server
      let db_client = DbClient::new("127.0.0.1:1369".to_string()).await?;


      let tracing_id: Option<String> = None;


      // Call info_server and print the result
      let info = db_client.info_server(tracing_id).await?;
      println!("Server info: {:?}", info);


      Ok(())
  }

  ```
</details>

## Parameters
* `tracing_id: Option<String>` — Optional tracing context propagated with the request.


## Behavior
* Sends a lightweight, read-only RPC; it has no side effects.

* Tracing metadata is attached when `tracing_id` is provided.

* Expects the response to include server info; the inner payload is unwrapped.
