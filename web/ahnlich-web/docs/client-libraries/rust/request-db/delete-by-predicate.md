---
title: Delete By Predicate
---

# Delete By Predicate

Removes one or more records from a store based on a predicate condition. Instead of targeting records by key, this operation evaluates a logical filter against stored metadata or values and deletes all matching entries. It provides a flexible way to bulk-remove records without knowing their exact keys.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_types::db::query::DelPred;
  use ahnlich_types::db::server::Del;
  use ahnlich_types::predicates::{Predicate, PredicateCondition, predicate::Kind as PredicateKind, predicate_condition::Kind as PredicateConditionKind};
  use ahnlich_client_rs::db::DbClient;
  use ahnlich_types::metadata::{MetadataValue, metadata_value::Value};
  use std::collections::HashMap;


  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      // gRPC server address
      let addr = "http://127.0.0.1:1369".to_string();
      let db_client = DbClient::new(addr).await?;


      // Define the predicate condition to delete
      let condition = PredicateCondition {
          kind: Some(PredicateConditionKind::Value(Predicate {
              kind: Some(PredicateKind::Equals(ahnlich_types::predicates::Equals {
                  key: "medal".into(),
                  value: Some(MetadataValue {
                      value: Some(Value::RawString("gold".into())),
                  }),
              })),
          })),
      };


      // Parameters for deleting predicate
      let params = DelPred {
          store: "Main".to_string(), // your store name
          condition: Some(condition),
      };


      // Call delete predicate
      match db_client.del_pred(params, None).await {
          Ok(Del { deleted_count }) => {
              println!("Successfully deleted {} predicate(s).", deleted_count);
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
* `Del` — Confirmation of the deletion request, including details of what was removed.

* `AhnlichError` — If the predicate is invalid or no matching records are found.


## Behavior
* Wraps the predicate parameters in a request object.

* Propagates tracing context if available.

* Executes the delete operation against the DB service.

* Returns confirmation once the matching records are deleted.
