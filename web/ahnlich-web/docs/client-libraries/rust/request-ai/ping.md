---
title: Ping
---

# Ping

Checks connectivity with the **Ahnlich AI** service and verifies the server is reachable over the current gRPC channel. Use this lightweight call for health checks or to validate that the AI client can communicate with the service before issuing embedding or inference requests.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  // src/bin/pingai.rs


  use ahnlich_client_rs::ai::AiClient; // AiClient path
  use ahnlich_client_rs::error::AhnlichError; // Error type
  use ahnlich_types::ai::pipeline::AiResponsePipeline;


  #[tokio::main]
  async fn main() -> Result<(), AhnlichError> {
      // AI server address
      let addr = "http://127.0.0.1:1370";


      // Initialize the AI client
      let ai_client = AiClient::new(addr.to_string()).await?;


      // Simple ping request
      let pong = ai_client.ping(None).await?;
      println!("AI Server Pong received: {:?}", pong);


      // Using a pipeline to send a ping
      let mut pipeline = ai_client.pipeline(None);
      pipeline.ping();
      let res: AiResponsePipeline = pipeline.exec().await?;
      println!("Pipeline response: {:?}", res);


      Ok(())
  }
  ```
</details>

## Parameters
* `tracing_id: Option<String>` — Optional trace parent used to propagate observability metadata with the request.


## Returns
* `Ok(Pong)` — A lightweight acknowledgement from the server indicating connectivity.

* `Err(AhnlichError)` — If the request fails due to transport, server, or authentication errors.


## Behavior (explains the code, brief)
* Constructs a `tonic::Request` carrying an empty `Ping {}` message.

* Calls `add_trace_parent(&mut req, tracing_id)` to attach the optional tracing context to the gRPC metadata.

* Uses a cloned gRPC client to call the remote `ping` RPC, awaits the response, and returns the inner `Pong` payload.
