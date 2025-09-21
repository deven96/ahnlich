---
title: Get By Predicate
---

# Get By Predicate

Retrieves records from a vector store that satisfy a given predicate filter. Supports filtered semantic queries where embeddings must meet metadata-based conditions in addition to similarity searches.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::ai::AiClient;
  use ahnlich_client_rs::error::AhnlichError;
  use ahnlich_client_rs::prelude::StoreName;
  use ahnlich_types::{
      ai::query::GetPred,
      predicates::{Predicate, PredicateCondition, predicate::Kind as PredicateKind, predicate_condition::Kind as PredicateConditionKind, Equals},
      metadata::{MetadataValue, metadata_value::Value as MValue},
  };
  use tokio;


  #[tokio::main]
  async fn main() -> Result<(), AhnlichError> {
      // Connect to AI server
      let ai_client = AiClient::new("http://127.0.0.1:1370".to_string())
          .await
          .expect("Failed to connect AI client");


      // Define store and metadata condition
      let store_name = StoreName { value: "Deven Kicks".to_string() };
      let matching_metadatakey = "Brand".to_string();
      let matching_metadatavalue = MetadataValue { value: Some(MValue::RawString("Nike".into())) };


      // Build predicate condition
      let condition = PredicateCondition {
          kind: Some(PredicateConditionKind::Value(Predicate {
              kind: Some(PredicateKind::Equals(Equals {
                  key: matching_metadatakey.clone(),
                  value: Some(matching_metadatavalue.clone()),
              })),
          })),
      };


      let get_pred_params = GetPred {
          store: store_name.value.clone(),
          condition: Some(condition),
      };


      // Call get_pred
      let response = ai_client.get_pred(get_pred_params, None).await?;


      println!("Matching entries:");
      for entry in response.entries {
          println!("{:?}", entry);
      }


      Ok(())
  }
  ```
</details>

## Parameters
* `params: GetPred` — Contains the store name and predicate expression used to filter records.

* `tracing_id: Option<String>` — Optional trace parent ID for distributed tracing and observability.


## Returns
* `Ok(Get)` — A collection of records (embeddings + metadata) that satisfy the predicate filter.

* `Err(AhnlichError)` — Returned if the store cannot be queried, the predicate is invalid, or the request fails.


## Behavior
* Constructs a `tonic::Request` from the `GetPred` parameters.

* Attaches tracing metadata when provided.

* Calls the AI service’s `get_pred` RPC endpoint.

* Waits for the server response and extracts the result.

* Returns the retrieved records packaged in a `Get` object.
