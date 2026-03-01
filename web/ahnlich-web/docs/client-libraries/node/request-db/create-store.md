---
title: Create Store
sidebar_position: 6
---

# Create Store

The CreateStore request creates a new vector store on the Ahnlich DB server.

* **Input**: Store name, dimension, optional predicates, and error handling flag.

* **Behavior**: Creates a new store with the specified configuration. The dimension is fixed at creation - all inserted vectors must match it.

* **Response**: Confirmation of store creation.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { CreateStore } from "ahnlich-client-node/grpc/db/query_pb";

async function createStore() {
  const client = createDbClient("127.0.0.1:1369");

  await client.createStore(
    new CreateStore({
      store: "my_store",
      dimension: 4,
      predicates: ["label", "category"],
      errorIfExists: true,
    })
  );

  console.log("Store created successfully");
}

createStore();
```
</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `store` | `string` | Yes | The name for the new store |
| `dimension` | `number` | Yes | Vector dimension (all vectors must match this) |
| `predicates` | `string[]` | No | List of predicate keys to index |
| `errorIfExists` | `boolean` | No | If `true`, throws error if store exists; if `false`, silently ignores |

## Example with All Options

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { CreateStore } from "ahnlich-client-node/grpc/db/query_pb";

async function createStoreWithOptions() {
  const client = createDbClient("127.0.0.1:1369");

  // Create a store for 128-dimensional embeddings
  // with predicate indices on "title" and "author"
  await client.createStore(
    new CreateStore({
      store: "book_embeddings",
      dimension: 128,
      predicates: ["title", "author", "genre"],
      errorIfExists: false, // Don't error if already exists
    })
  );
}

createStoreWithOptions();
```
</details>

## Notes

- Store dimension cannot be changed after creation
- Predicate indices can be added later using [Create Predicate Index](/docs/client-libraries/node/request-db/create-predicate-index)
- Non-linear indices (KDTree, HNSW) can be added using [Create Non Linear Algorithm Index](/docs/client-libraries/node/request-db/create-non-linear-algx)
