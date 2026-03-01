---
title: GetSimN
sidebar_position: 7
---

# GetSimN

The GetSimN request performs semantic similarity search, finding entries most similar to a query input.

* **Input**: Store name, query input, number of results, and similarity algorithm.

* **Behavior**: Embeds the query using the store's query model and finds the N most similar entries.

* **Response**: A list of entries with their similarity scores.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createAiClient } from "ahnlich-client-node";
import { GetSimN } from "ahnlich-client-node/grpc/ai/query_pb";
import { StoreInput } from "ahnlich-client-node/grpc/keyval_pb";
import { Algorithm } from "ahnlich-client-node/grpc/algorithm/algorithm_pb";

async function getSimN() {
  const client = createAiClient("127.0.0.1:1370");

  const response = await client.getSimN(
    new GetSimN({
      store: "ai_store",
      searchInput: new StoreInput({ value: { case: "rawString", value: "Jordan" } }),
      closestN: 3,
      algorithm: Algorithm.COSINE_SIMILARITY,
    })
  );

  console.log(response.entries);

  for (const entry of response.entries) {
    console.log(`Input: ${entry.input?.value}`);
    console.log(`Similarity: ${entry.similarity}`);
    console.log(`Metadata: ${JSON.stringify(entry.value?.value)}`);
  }
}

getSimN();
```
</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `store` | `string` | Yes | The name of the AI store |
| `searchInput` | `StoreInput` | Yes | The query input (text or image) |
| `closestN` | `number` | Yes | Number of similar entries to return |
| `algorithm` | `Algorithm` | Yes | Similarity algorithm to use |
| `condition` | `PredicateCondition` | No | Optional filter condition |

## Example with Text Search

<details>
  <summary>Click to expand source code</summary>

```ts
import { createAiClient } from "ahnlich-client-node";
import { GetSimN } from "ahnlich-client-node/grpc/ai/query_pb";
import { StoreInput } from "ahnlich-client-node/grpc/keyval_pb";
import { Algorithm } from "ahnlich-client-node/grpc/algorithm/algorithm_pb";

async function semanticSearch() {
  const client = createAiClient("127.0.0.1:1370");

  // Search for shoes similar to "comfortable running shoes"
  const response = await client.getSimN(
    new GetSimN({
      store: "products",
      searchInput: new StoreInput({
        value: { case: "rawString", value: "comfortable running shoes" },
      }),
      closestN: 5,
      algorithm: Algorithm.COSINE_SIMILARITY,
    })
  );

  console.log("Top 5 similar products:");
  for (const entry of response.entries) {
    console.log(`- ${entry.input?.value} (similarity: ${entry.similarity})`);
  }
}

semanticSearch();
```
</details>

## Example with Predicate Filter

<details>
  <summary>Click to expand source code</summary>

```ts
import { createAiClient } from "ahnlich-client-node";
import { GetSimN } from "ahnlich-client-node/grpc/ai/query_pb";
import { StoreInput } from "ahnlich-client-node/grpc/keyval_pb";
import { Algorithm } from "ahnlich-client-node/grpc/algorithm/algorithm_pb";
import { PredicateCondition, Predicate, Equals } from "ahnlich-client-node/grpc/predicate_pb";
import { MetadataValue } from "ahnlich-client-node/grpc/metadata_pb";

async function searchWithFilter() {
  const client = createAiClient("127.0.0.1:1370");

  // Search for Nike products similar to "basketball shoes"
  const response = await client.getSimN(
    new GetSimN({
      store: "products",
      searchInput: new StoreInput({
        value: { case: "rawString", value: "basketball shoes" },
      }),
      closestN: 5,
      algorithm: Algorithm.COSINE_SIMILARITY,
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

  console.log("Top Nike basketball-related products:");
  for (const entry of response.entries) {
    console.log(`- ${entry.input?.value}`);
  }
}

searchWithFilter();
```
</details>

## Notes

- The query input is automatically embedded using the store's query model
- Semantic search finds conceptually similar results, not just keyword matches
- Use predicate filters to narrow results by metadata
