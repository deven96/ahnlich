---
title: Create Predicate Index
sidebar_position: 11
---

# Create Predicate Index

The CreatePredIndex request creates an index on metadata fields to optimize predicate-based queries.

* **Input**: Store name and list of predicate keys to index.

* **Behavior**: Creates indices on the specified metadata fields for faster filtering.

* **Response**: Confirmation of index creation.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { CreatePredIndex } from "ahnlich-client-node/grpc/db/query_pb";

async function createPredicateIndex() {
  const client = createDbClient("127.0.0.1:1369");

  await client.createPredIndex(
    new CreatePredIndex({
      store: "my_store",
      predicates: ["label", "category"],
    })
  );

  console.log("Predicate indices created successfully");
}

createPredicateIndex();
```
</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `store` | `string` | Yes | The name of the store |
| `predicates` | `string[]` | Yes | List of metadata keys to index |

## Notes

- Indices significantly speed up predicate-based queries ([GetPred](/docs/client-libraries/node/request-db/get-by-predicate), filtered [GetSimN](/docs/client-libraries/node/request-db/get-simn))
- Creating indices has a one-time cost but improves query performance
- Indices can also be specified during [store creation](/docs/client-libraries/node/request-db/create-store)
