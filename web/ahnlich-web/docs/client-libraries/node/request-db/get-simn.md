---
title: GetSimN
sidebar_position: 8
---

# GetSimN

The GetSimN request finds the N closest (most similar) entries to a query vector using the specified similarity algorithm.

* **Input**: Store name, query vector, number of results, and similarity algorithm.

* **Behavior**: Performs a similarity search and returns the closest N entries.

* **Response**: A list of entries with their similarity scores.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { GetSimN } from "ahnlich-client-node/grpc/db/query_pb";
import { StoreKey } from "ahnlich-client-node/grpc/keyval_pb";
import { Algorithm } from "ahnlich-client-node/grpc/algorithm/algorithm_pb";

async function getSimN() {
  const client = createDbClient("127.0.0.1:1369");

  const response = await client.getSimN(
    new GetSimN({
      store: "my_store",
      searchInput: new StoreKey({ key: [1.0, 2.0, 3.0, 4.0] }),
      closestN: 3,
      algorithm: Algorithm.COSINE_SIMILARITY,
    })
  );

  console.log(response.entries);

  // Iterate over results
  for (const entry of response.entries) {
    console.log(`Key: ${entry.key?.key}`);
    console.log(`Similarity: ${entry.similarity}`);
    console.log(`Value: ${JSON.stringify(entry.value?.value)}`);
  }
}

getSimN();
```
</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `store` | `string` | Yes | The name of the store |
| `searchInput` | `StoreKey` | Yes | The query vector |
| `closestN` | `number` | Yes | Number of similar entries to return |
| `algorithm` | `Algorithm` | Yes | Similarity algorithm to use |
| `condition` | `PredicateCondition` | No | Optional filter condition |

## Available Algorithms

| Algorithm | Description |
|-----------|-------------|
| `Algorithm.COSINE_SIMILARITY` | Cosine similarity (good for text embeddings) |
| `Algorithm.EUCLIDEAN_DISTANCE` | Euclidean distance (L2 norm) |
| `Algorithm.DOT_PRODUCT` | Dot product similarity |

## Example with Predicate Filter

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { GetSimN } from "ahnlich-client-node/grpc/db/query_pb";
import { StoreKey } from "ahnlich-client-node/grpc/keyval_pb";
import { Algorithm } from "ahnlich-client-node/grpc/algorithm/algorithm_pb";
import { PredicateCondition, Predicate, Equals } from "ahnlich-client-node/grpc/predicate_pb";
import { MetadataValue } from "ahnlich-client-node/grpc/metadata_pb";

async function getSimNWithFilter() {
  const client = createDbClient("127.0.0.1:1369");

  const response = await client.getSimN(
    new GetSimN({
      store: "my_store",
      searchInput: new StoreKey({ key: [1.0, 2.0, 3.0, 4.0] }),
      closestN: 5,
      algorithm: Algorithm.COSINE_SIMILARITY,
      condition: new PredicateCondition({
        kind: {
          case: "value",
          value: new Predicate({
            kind: {
              case: "equals",
              value: new Equals({
                key: "category",
                value: new MetadataValue({ value: { case: "rawString", value: "electronics" } }),
              }),
            },
          }),
        },
      }),
    })
  );

  console.log(`Found ${response.entries.length} similar entries in category 'electronics'`);
}

getSimNWithFilter();
```
</details>

## Notes

- The query vector dimension must match the store dimension
- Non-linear indices (KDTree, HNSW) can significantly speed up searches on large stores
- When using predicate filters, ensure the filter key has a predicate index for optimal performance
