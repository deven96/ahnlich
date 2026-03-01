---
title: Get Store
sidebar_position: 5
---

# Get Store

The GetStore request retrieves detailed information about a single store by its name.

* **Input**: Store name.

* **Behavior**: The server returns detailed information about the specified store.

* **Response**: Store information including name, dimension, indices, and size.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { GetStore } from "ahnlich-client-node/grpc/db/query_pb";

async function getStore() {
  const client = createDbClient("127.0.0.1:1369");

  const response = await client.getStore(
    new GetStore({ store: "my_store" })
  );

  console.log(response.name);             // Store name
  console.log(response.dimension);         // Vector dimension
  console.log(response.predicateIndices);  // Indexed predicate keys
  console.log(response.nonLinearIndices);  // Non-linear algorithm indices
  console.log(response.len);              // Number of entries
  console.log(response.sizeInBytes);      // Size on disk
}

getStore();
```
</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `store` | `string` | Yes | The name of the store to retrieve |

## Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | `string` | The name of the store |
| `dimension` | `number` | Vector dimension for this store |
| `len` | `number` | Number of entries in the store |
| `sizeInBytes` | `bigint` | Total size of the store in bytes |
| `predicateIndices` | `string[]` | List of indexed predicate keys |
| `nonLinearIndices` | `NonLinearAlgorithm[]` | List of non-linear indices |
