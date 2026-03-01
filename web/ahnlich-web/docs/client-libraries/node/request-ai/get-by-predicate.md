---
title: Get By Predicate
sidebar_position: 8
---

# Get By Predicate

The GetPred request retrieves entries from an AI store that match a specified predicate condition.

* **Input**: Store name and predicate condition.

* **Behavior**: Returns all entries whose metadata matches the predicate condition.

* **Response**: A list of matching entries.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createAiClient } from "ahnlich-client-node";
import { GetPred } from "ahnlich-client-node/grpc/ai/query_pb";
import { PredicateCondition, Predicate, Equals } from "ahnlich-client-node/grpc/predicate_pb";
import { MetadataValue } from "ahnlich-client-node/grpc/metadata_pb";

async function getByPredicate() {
  const client = createAiClient("127.0.0.1:1370");

  const response = await client.getPred(
    new GetPred({
      store: "ai_store",
      condition: new PredicateCondition({
        kind: {
          case: "value",
          value: new Predicate({
            kind: {
              case: "equals",
              value: new Equals({
                key: "brand",
                value: new MetadataValue({ value: { case: "rawString", value: "Nike" } }),
              }),
            },
          }),
        },
      }),
    })
  );

  console.log(`Found ${response.entries.length} Nike products`);
}

getByPredicate();
```
</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `store` | `string` | Yes | The name of the AI store |
| `condition` | `PredicateCondition` | Yes | The filter condition |

## Notes

- Same predicate structure as DB operations
- For optimal performance, create predicate indices on frequently filtered fields
- See [Type Meanings](/docs/client-libraries/node/type-meanings) for predicate details
