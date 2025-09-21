---
title: Get By Predicate
---

# Get By Predicate

Retrieve one or more stored vectors and their associated metadata from a store by applying a predicate filter. Unlike `Get Key`, which retrieves a single item by its unique key, `Get by Predicate` allows querying based on conditions defined on metadata fields (for example, "all items where `category = book`"). This is useful for flexible filtering, targeted queries, or conditional retrieval.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::db::DbClient;
  use ahnlich_types::{
      db::query::GetPred,
      metadata::{MetadataValue, metadata_value::Value},
      predicates::{
          Predicate, PredicateCondition,
          predicate::Kind as PredicateKind,
          predicate_condition::Kind as PredicateConditionKind,
          Equals,
      },
  };
  use tokio;


  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      let addr = "http://127.0.0.1:1369";
      let client = DbClient::new(addr.to_string()).await?;


      let condition = PredicateCondition {
          kind: Some(PredicateConditionKind::Value(Predicate {
              kind: Some(PredicateKind::Equals(Equals {
                  key: "role".into(),
                  value: Some(MetadataValue {
                      value: Some(Value::RawString("admin".into())),
                  }),
              })),
          })),
      };


      let get_pred_params = GetPred {
          store: "Main".to_string(),
          condition: Some(condition),
      };


      let result = client.get_pred(get_pred_params, None).await?;
      println!("Fetched rows: {:#?}", result);


      Ok(())
  }

  ```
</details>

## Parameters
* `params: GetPred` — Contains the store name and predicate condition used to filter results.

* `tracing_id: Option<String>` — Optional trace context for observability, attached to the request if provided.


## Returns
* `Get` — Response with all matched vectors and metadata.

* `AhnlichError` — Error if the store is missing, the predicate is invalid, or the server cannot complete the request.


## Behavior
* Builds a gRPC request from the given `GetPred` parameters.

* Attaches optional tracing metadata using `add_trace_parent`.

* Calls the `get_pred` RPC on the server through the cloned client.

* Waits asynchronously for the server’s response and extracts the inner `Get` payload with `.into_inner()`.
