---
title: Info Server
---

# Info Server

Retrieves detailed information about the Ahnlich AI service server, including metadata such as version, build information, and runtime configuration. This call is useful for diagnostics, compatibility checks, and ensuring the AI service is running as expected.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::ai::AiClient; // <-- note the `ai::` path
  use ahnlich_client_rs::error::AhnlichError;
  use ahnlich_types::shared::info::ServerInfo;


  #[tokio::main]
  async fn main() -> Result<(), AhnlichError> {
      let addr = "127.0.0.1:1370";
      let client = AiClient::new(addr.to_string()).await?;


      // Direct info_server call
      let server_info: ServerInfo = client.info_server(None).await?;
      println!("Server Info: {:?}", server_info);


      // Using pipeline
      let mut pipeline = client.pipeline(None);
      pipeline.info_server();
      let pipeline_result = pipeline.exec().await?;
      println!("Pipeline Server Info: {:?}", pipeline_result);


      Ok(())
  }
  ```
</details>

## Parameters
* `tracing_id: Option<String>` — Optional trace parent attached to propagate observability metadata with the request.


## Returns
* `Ok(ServerInfo)` — Metadata describing the AI service server (e.g., version, configuration).

* `Err(AhnlichError)` — If the server cannot be reached, request fails, or metadata is missing.


## Behavior (explains the code, brief)
* Builds a `tonic::Request` with an empty `InfoServer {}` message.

* Propagates tracing context if provided.

* Calls the remote `info_server` RPC using the cloned client.

* Awaits and unwraps the server response.

* Extracts the `info` field, returning it as `ServerInfo`.

* Ensures the `info` is not `None` with `.expect()`, which will panic if missing (a server contract guarantee).
