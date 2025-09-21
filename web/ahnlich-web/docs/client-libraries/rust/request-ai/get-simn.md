---
title: Get Sim N
---

# Get Sim N

Performs a **similarity search** in a vector store, retrieving the top-N most similar embeddings to a given query vector. This operation is the core of the AI client’s retrieval capability, enabling semantic search, recommendation, and nearest-neighbor lookups.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::ai::AiClient;
  use ahnlich_client_rs::error::AhnlichError;
  use ahnlich_types::ai::preprocess::PreprocessAction;
  use ahnlich_types::ai::query::GetSimN;
  use ahnlich_types::keyval::{StoreInput};
  use ahnlich_types::keyval::store_input::Value; // Correct path


  #[tokio::main]
  async fn main() -> Result<(), AhnlichError> {
      // Connect to AI server
      let addr = "127.0.0.1:1370";
      let client = AiClient::new(addr.to_string()).await?;


      // Prepare the search input
      let search_input = StoreInput {
          value: Some(Value::RawString("example query".into())),
      };


      // Construct GetSimN parameters
      let params = GetSimN {
          store: "Main0".to_string(),
          search_input: Some(search_input),
          closest_n: 3, // number of similar entries to retrieve
          algorithm: 0, // default algorithm (0 usually corresponds to Cosine)
          execution_provider: None,
          preprocess_action: PreprocessAction::NoPreprocessing as i32,
          condition: None,
      };


      // Run the GetSimN command
      let res = client.get_sim_n(params, None).await?;
      println!("GetSimN result: {:?}", res);


      Ok(())
  }
  ```
</details>

## Parameters
* `params: GetSimN` — The query input, including the target vector and configuration such as the number of neighbors (`N`) to return and optional filters.


* `tracing_id: Option<String>` — Optional trace parent ID for distributed observability across services.


## Returns
* `Ok(GetSimNResult)` — Contains the ranked list of the top-N closest embeddings along with their similarity scores and associated metadata.

* `Err(AhnlichError)` — Returned when the query fails, parameters are invalid, or the service encounters an error.


## Behavior (explains the code, brief)
* Wraps the `GetSimN` query in a `tonic::Request`.

* Adds trace propagation metadata if provided.

* Sends the request to the AI service’s `get_sim_n` RPC method.

* Waits for the response and unwraps the result.

* Returns a `GetSimNResult` with the similarity matches.
