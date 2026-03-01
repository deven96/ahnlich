---
title: Drop Predicate Index
sidebar_position: 12
---

# Drop Predicate Index

The DropPredIndex request removes an existing predicate index from a store.

* **Input**: Store name, list of predicate keys to remove, and error handling flag.

* **Behavior**: Removes the specified predicate indices.

* **Response**: Confirmation of index removal.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { DropPredIndex } from "ahnlich-client-node/grpc/db/query_pb";

async function dropPredicateIndex() {
  const client = createDbClient("127.0.0.1:1369");

  await client.dropPredIndex(
    new DropPredIndex({
      store: "my_store",
      predicates: ["label"],
      errorIfNotExists: true,
    })
  );

  console.log("Predicate index dropped successfully");
}

dropPredicateIndex();
```
</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `store` | `string` | Yes | The name of the store |
| `predicates` | `string[]` | Yes | List of predicate indices to remove |
| `errorIfNotExists` | `boolean` | No | If `true`, throws error if index doesn't exist |

## Notes

- Dropping an index does not delete the underlying data
- After dropping, predicate queries on that field will be slower (full scan)
- Consider keeping indices for frequently filtered fields
