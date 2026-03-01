---
title: Create Non Linear Algorithm Index
sidebar_position: 13
---

# Create Non Linear Algorithm Index

The CreateNonLinearAlgorithmIndex request creates a non-linear index (KDTree or HNSW) to speed up similarity searches.

* **Input**: Store name and list of non-linear indices to create.

* **Behavior**: Builds the specified non-linear index structure for faster similarity queries.

* **Response**: Confirmation of index creation.

## Create a KDTree Index

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { CreateNonLinearAlgorithmIndex } from "ahnlich-client-node/grpc/db/query_pb";
import { NonLinearIndex, KDTreeConfig } from "ahnlich-client-node/grpc/algorithm/nonlinear_pb";

async function createKDTreeIndex() {
  const client = createDbClient("127.0.0.1:1369");

  await client.createNonLinearAlgorithmIndex(
    new CreateNonLinearAlgorithmIndex({
      store: "my_store",
      nonLinearIndices: [
        new NonLinearIndex({
          index: { case: "kdtree", value: new KDTreeConfig() },
        }),
      ],
    })
  );

  console.log("KDTree index created successfully");
}

createKDTreeIndex();
```
</details>

## Create an HNSW Index

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { CreateNonLinearAlgorithmIndex } from "ahnlich-client-node/grpc/db/query_pb";
import { NonLinearIndex, HNSWConfig } from "ahnlich-client-node/grpc/algorithm/nonlinear_pb";

async function createHNSWIndex() {
  const client = createDbClient("127.0.0.1:1369");

  await client.createNonLinearAlgorithmIndex(
    new CreateNonLinearAlgorithmIndex({
      store: "my_store",
      nonLinearIndices: [
        new NonLinearIndex({
          index: { case: "hnsw", value: new HNSWConfig() },
        }),
      ],
    })
  );

  console.log("HNSW index created successfully");
}

createHNSWIndex();
```
</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `store` | `string` | Yes | The name of the store |
| `nonLinearIndices` | `NonLinearIndex[]` | Yes | List of indices to create |

## Index Types

| Type | Description | Best For |
|------|-------------|----------|
| `KDTree` | K-dimensional tree | Lower dimensions (less than 20), exact searches |
| `HNSW` | Hierarchical Navigable Small World | High dimensions, approximate but fast |

## Notes

- Non-linear indices dramatically improve [GetSimN](/docs/client-libraries/node/request-db/get-simn) performance on large stores
- HNSW is generally recommended for high-dimensional embeddings (128+)
- Building indices takes time and memory proportional to store size
- You can have multiple index types on the same store
