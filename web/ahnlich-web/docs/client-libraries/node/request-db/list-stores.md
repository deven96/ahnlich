---
title: List Stores
sidebar_position: 4
---

# List Stores

The ListStores request retrieves a list of all vector stores available on the Ahnlich DB server.

* **Input**: No arguments required.

* **Behavior**: The server returns information about all existing stores including their names, dimensions, and indices.

* **Response**: A list of `StoreInfo` objects containing store metadata.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { ListStores } from "ahnlich-client-node/grpc/db/query_pb";

async function listStores() {
  const client = createDbClient("127.0.0.1:1369");

  const response = await client.listStores(new ListStores());

  // Get store names
  console.log(response.stores.map((s) => s.name));

  // Iterate over stores with full details
  for (const store of response.stores) {
    console.log(`Store: ${store.name}`);
    console.log(`  Dimension: ${store.dimension}`);
    console.log(`  Entries: ${store.len}`);
    console.log(`  Size: ${store.sizeInBytes} bytes`);
    console.log(`  Predicate Indices: ${store.predicateIndices}`);
    console.log(`  Non-Linear Indices: ${store.nonLinearIndices}`);
  }
}

listStores();
```
</details>

## StoreInfo Fields

Each `StoreInfo` object contains:

| Field | Type | Description |
|-------|------|-------------|
| `name` | `string` | The name of the store |
| `dimension` | `number` | Vector dimension for this store |
| `len` | `number` | Number of entries in the store |
| `sizeInBytes` | `bigint` | Total size of the store in bytes |
| `predicateIndices` | `string[]` | List of indexed predicate keys |
| `nonLinearIndices` | `NonLinearAlgorithm[]` | List of non-linear indices (KDTree, HNSW) |
