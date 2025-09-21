---
title: Pipeline
---

# Pipeline

Creates a new **AI pipeline** for batching multiple operations in the **AI client**. Pipelines allow you to queue multiple requests (e.g., `set`, `get_sim_n`, `get_key`) and execute them in sequence, ensuring consistent ordering and efficient handling of AI service calls. This is particularly useful for bulk processing or workflows that require multiple embeddings or queries to be executed together.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::ai::{AiClient, AiPipeline};
use ahnlich_client_rs::error::AhnlichError;
  use tokio;


  #[tokio::main]
  async fn main() -> Result<(), AhnlichError> {
      // Replace with your AI server address
      let addr = "127.0.0.1:1370".to_string();


      // Initialize the client
      let client = AiClient::new(addr).await?;


      // Create a pipeline
      let mut pipeline = client.pipeline(None);


      // Add some example queries
      pipeline.ping();      // Ping the server
      pipeline.list_stores(); // List existing stores


      // Execute the pipeline
      let response = pipeline.exec().await?;


      println!("Pipeline response: {:#?}", response);


      Ok(())
  }
  ```
</details>

## Parameters
* `tracing_id: Option<String>` — Optional trace parent ID for observability and distributed tracing.


## Returns
* `AiPipeline` — A new pipeline instance that can queue AI client operations and execute them sequentially.


## Behavior (explains the code, brief)
* Initializes an empty `queries` vector to hold pending operations.

* Clones the AI client for use within the pipeline.

* Attaches optional tracing metadata for observability.

* Returns a fully initialized `AiPipeline` object ready for queuing operations.
