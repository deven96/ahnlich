---
title: Delete By Predicate
---

# Delete By Predicate

Removes one or more records from an AI store based on a predicate condition. This is a passthrough operation to the underlying DB service. Instead of targeting records by key, this operation evaluates a logical filter against stored metadata or values and deletes all matching entries. It provides a flexible way to bulk-remove records without knowing their exact keys.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_types::ai::query::DelPred;
  use ahnlich_types::ai::server::Del;
  use ahnlich_types::predicates::{Predicate, PredicateCondition, predicate::Kind as PredicateKind, predicate_condition::Kind as PredicateConditionKind};
  use ahnlich_client_rs::ai::AiClient;
  use ahnlich_types::metadata::{MetadataValue, metadata_value::Value};
  use std::collections::HashMap;


  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      // gRPC server address
      let addr = "http://127.0.0.1:1370".to_string();
      let ai_client = AiClient::new(addr).await?;


      // Define the predicate condition to delete
      let condition = PredicateCondition {
          kind: Some(PredicateConditionKind::Value(Predicate {
              kind: Some(PredicateKind::Equals(ahnlich_types::predicates::Equals {
                  key: "category".into(),
                  value: Some(MetadataValue {
                      value: Some(Value::RawString("archived".into())),
                  }),
              })),
          })),
      };


      // Parameters for deleting predicate
      let params = DelPred {
          store: "my_ai_store".to_string(),
          condition: Some(condition),
      };


      // Call delete predicate
      match ai_client.del_pred(params, None).await {
          Ok(Del { deleted_count }) => {
              println!("Successfully deleted {} record(s).", deleted_count);
          }
          Err(e) => {
              eprintln!("Error deleting predicate: {:?}", e);
          }
      }


      Ok(())
  }
  ```
</details>

## Parameters
* `params: DelPred` — Defines the predicate filter used to match records for deletion.

* `tracing_id: Option<String>` — Optional trace identifier for monitoring and observability.


## Returns
* `Del` — Confirmation of the deletion request, including the count of deleted items.

* `AhnlichError` — If the predicate is invalid or the operation fails.


## Behavior
* This is a passthrough operation to the underlying DB service.

* Wraps the predicate parameters in a request object.

* Propagates tracing context if available.

* Executes the delete operation against the AI service.

* Returns confirmation once the matching records are deleted.
