---
title: Upsert
---

# Upsert

## Schema

This request accepts an optional `schema` field. When it is omitted, the server uses the `public` schema. Set `schema` to target a store in another schema.

The Upsert request updates a single entry matching a predicate condition in an AI store. The AI service automatically merges metadata and can re-embed new inputs.

* **Input**: Store name, predicate condition, optional new input/value, preprocessing options.

* **Behavior**: Updates exactly one matching entry. AI proxy always merges metadata. Errors on 0 or multiple matches.

* **Response**: Upsert counts (inserted and updated).

<details>
  <summary>Click to expand source code</summary>

```ts
import { createAiClient } from "ahnlich-client-node";
import { Upsert } from "ahnlich-client-node/grpc/ai/query_pb";
import { StoreValue } from "ahnlich-client-node/grpc/keyval_pb";
import { MetadataValue } from "ahnlich-client-node/grpc/metadata_pb";
import { PredicateCondition, Predicate, Equals } from "ahnlich-client-node/grpc/predicates_pb";
import { PreprocessAction } from "ahnlich-client-node/grpc/ai/preprocess_pb";

async function upsertEntry() {
  const client = createAiClient("127.0.0.1:1370");

  const condition = new PredicateCondition({
    kind: {
      case: "value",
      value: new Predicate({
        kind: {
          case: "equals",
          value: new Equals({
            key: "filename",
            value: new MetadataValue({ value: { case: "rawString", value: "photo.jpg" } }),
          }),
        },
      }),
    },
  });

  const newValue = new StoreValue({
    value: {
      tags: new MetadataValue({ value: { case: "rawString", value: "cat,outdoors" } }),
    },
  });

  const response = await client.upsert(
    new Upsert({
      store: "images",
      schema: "media",
      condition,
      newValue,
      preprocessAction: PreprocessAction.NO_PREPROCESSING,
    })
  );

  console.log("Updated:", response.upsert?.updated);
}

upsertEntry();
```
</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `store` | `string` | Yes | The name of the AI store |
| `condition` | `PredicateCondition` | Yes | Must match exactly one entry |
| `newInput` | `StoreInput` | No | New input (text, image, or audio) to re-embed |
| `newValue` | `StoreValue` | No | Metadata to update (always merged) |
| `preprocessAction` | `PreprocessAction` | No | Preprocessing for new input |
| `executionProvider` | `ExecutionProvider` | No | Hardware acceleration (e.g., CUDA) |
| `modelParams` | `Record<string, string>` | No | Runtime model parameters |
| `schema` | `string` | No | Schema namespace (defaults to "public") |

## Behavior

- AI proxy always merges metadata (preserves AI-generated fields)
- Predicate must match exactly one entry
- Re-embeds input if `newInput` is provided
- Errors if 0 or multiple entries match
- Updated entries are immediately available for similarity search
