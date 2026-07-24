---
title: Upsert
---

# Upsert

## Schema

This request accepts an optional `schema` field. When it is omitted, the server uses the `public` schema. Set `schema` to target a store in another schema.

The `Upsert` request updates a single entry that matches a predicate condition. It errors if the predicate matches 0 or multiple entries.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::db::DbClient;
  use ahnlich_types::{
      db::query::Upsert,
      keyval::{StoreKey, StoreValue},
      metadata::{MetadataValue, metadata_value::Value},
      predicates::{Predicate, PredicateCondition, predicate_condition::Kind, predicate::Kind as PredKind},
  };
  use std::collections::HashMap;


  #[tokio::main]
  async fn main() -> Result<(), Box<dyn std::error::Error>> {
      // Connect to DB server
      let db_client = DbClient::new("127.0.0.1:1369".to_string()).await?;

      let tracing_id: Option<String> = None;

      // Construct predicate condition
      let condition = PredicateCondition {
          kind: Some(Kind::Value(Predicate {
              kind: Some(PredKind::Equals(ahnlich_types::predicates::Equals {
                  key: "id".to_string(),
                  value: Some(MetadataValue {
                      value: Some(Value::RawString("123".to_string())),
                  }),
              })),
          })),
      };

      // New metadata to merge/replace
      let new_value = Some(StoreValue {
          value: HashMap::from_iter([(
              "status".into(),
              MetadataValue {
                  value: Some(Value::RawString("published".into())),
              },
          )]),
      });

      let params = Upsert {
          store: "my_store".to_string(),
          schema: Some("analytics".to_string()),
          condition: Some(condition),
          new_key: None, // Optional: new vector
          new_value,
          merge_metadata: true, // Merge instead of replace
      };

      // Call upsert
      match db_client.upsert(params, tracing_id).await {
          Ok(result) => {
              println!("Upsert result: {:?}", result);
          }
          Err(err) => {
              eprintln!("Error in upsert: {:?}", err);
          }
      }

      Ok(())
  }
  ```
</details>

## Parameters
* `params: Upsert` – Defines the update operation:

  * `store` – The name of the target store.

  * `condition` – Predicate that must match exactly one entry.

  * `new_key` (optional) – New vector to replace existing key.

  * `new_value` (optional) – Metadata to update.

  * `merge_metadata` – If true, merges new metadata into existing. If false, replaces entirely.

  * `schema` (optional) – Schema namespace (defaults to "public").

* `tracing_id: Option<String>` – An optional tracing identifier.

## Returns
* `Ok(Set)` – Contains upsert counts (inserted and updated).

* `Err(AhnlichError)` – Returned if:

  * Predicate matches 0 or multiple entries.

  * Store does not exist.

  * Invalid vector dimension.

  * Transport or server-side errors.

## Behavior
* Predicate must match exactly one entry (errors on 0 or multiple matches).

* `merge_metadata: true` merges new metadata into existing fields.

* `merge_metadata: false` replaces metadata entirely.

* Updated entries are immediately available for queries.
