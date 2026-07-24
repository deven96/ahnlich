---
title: Upsert
---

# Upsert

## Schema

This request accepts an optional `schema` field. When it is omitted, the server uses the `public` schema. Set `schema` to target a store in another schema.

Updates a single entry matching a predicate condition in an AI-powered store. The AI service automatically merges metadata and can re-embed new inputs.

## Source Code Example

<details>
  <summary>Click to expand</summary>

  ```rust
  use ahnlich_client_rs::ai::AiClient;
  use ahnlich_client_rs::error::AhnlichError;
  use ahnlich_types::ai::preprocess::PreprocessAction;
  use ahnlich_types::ai::query::Upsert;
  use ahnlich_types::keyval::{StoreInput, StoreValue};
  use ahnlich_types::metadata::{MetadataValue, metadata_value::Value};
  use ahnlich_types::predicates::{Predicate, PredicateCondition, predicate_condition::Kind, predicate::Kind as PredKind};
  use std::collections::HashMap;


  #[tokio::main]
  async fn main() -> Result<(), AhnlichError> {
      // Connect to AI server
      let addr = "127.0.0.1:1370";
      let client = AiClient::new(addr.to_string()).await?;

      // Construct predicate condition
      let condition = PredicateCondition {
          kind: Some(Kind::Value(Predicate {
              kind: Some(PredKind::Equals(ahnlich_types::predicates::Equals {
                  key: "filename".to_string(),
                  value: Some(MetadataValue {
                      value: Some(Value::RawString("photo.jpg".to_string())),
                  }),
              })),
          })),
      };

      // New metadata to merge
      let new_value = Some(StoreValue {
          value: HashMap::from_iter([(
              "tags".into(),
              MetadataValue {
                  value: Some(Value::RawString("cat,outdoors".into())),
              },
          )]),
      });

      let params = Upsert {
          store: "images".to_string(),
          schema: Some("media".to_string()),
          condition: Some(condition),
          new_input: None, // Optional: new image/text to re-embed
          new_value,
          preprocess_action: PreprocessAction::NoPreprocessing as i32,
          execution_provider: None,
          model_params: HashMap::new(),
      };

      // Run the upsert command
      let res = client.upsert(params, None).await?;
      println!("Upsert result: {:?}", res.upsert);

      Ok(())
  }
  ```
</details>

## Parameters
* `params: Upsert` — The update operation:

  * `store` — The name of the target store.

  * `condition` — Predicate that must match exactly one entry.

  * `new_input` (optional) — New raw input (text, image, or audio) to re-embed.

  * `new_value` (optional) — Metadata to update (always merged by AI proxy).

  * `preprocess_action` — How to preprocess new input.

  * `execution_provider` (optional) — Hardware acceleration (e.g., CUDA).

  * `model_params` — Optional runtime parameters for the AI model.

  * `schema` (optional) — Schema namespace (defaults to "public").

* `tracing_id: Option<String>` — An optional tracing identifier.

## Returns
* `Ok(Set)` — Contains upsert counts (inserted and updated).

* `Err(AhnlichError)` — Returned if:

  * Predicate matches 0 or multiple entries.

  * Store does not exist.

  * Invalid input format.

  * Transport or server-side errors.

## Behavior
* AI proxy always merges metadata (preserves AI-generated fields).

* Predicate must match exactly one entry (errors on 0 or multiple matches).

* Re-embeds input if `new_input` is provided.

* Updated entries are immediately available for similarity search.
