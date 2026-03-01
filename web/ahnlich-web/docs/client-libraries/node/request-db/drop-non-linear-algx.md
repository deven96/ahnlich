---
title: Drop Non Linear Algorithm Index
sidebar_position: 14
---

# Drop Non Linear Algorithm Index

The DropNonLinearAlgorithmIndex request removes a non-linear index (KDTree or HNSW) from a store.

* **Input**: Store name, list of index types to remove, and error handling flag.

* **Behavior**: Removes the specified non-linear indices.

* **Response**: Confirmation of index removal.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { DropNonLinearAlgorithmIndex } from "ahnlich-client-node/grpc/db/query_pb";
import { NonLinearAlgorithm } from "ahnlich-client-node/grpc/algorithm/nonlinear_pb";

async function dropNonLinearIndex() {
  const client = createDbClient("127.0.0.1:1369");

  await client.dropNonLinearAlgorithmIndex(
    new DropNonLinearAlgorithmIndex({
      store: "my_store",
      nonLinearIndices: [NonLinearAlgorithm.KDTree],
      errorIfNotExists: true,
    })
  );

  console.log("KDTree index dropped successfully");
}

dropNonLinearIndex();
```
</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `store` | `string` | Yes | The name of the store |
| `nonLinearIndices` | `NonLinearAlgorithm[]` | Yes | List of index types to remove |
| `errorIfNotExists` | `boolean` | No | If `true`, throws error if index doesn't exist |

## Available NonLinearAlgorithm Values

| Value | Description |
|-------|-------------|
| `NonLinearAlgorithm.KDTree` | K-dimensional tree |
| `NonLinearAlgorithm.HNSW` | Hierarchical Navigable Small World |

## Notes

- Dropping an index does not delete the underlying data
- Similarity searches will fall back to linear scan after dropping indices
- You may want to rebuild indices with different configurations
