---
title: Drop Store
sidebar_position: 17
---

# Drop Store

The DropStore request deletes an entire store and all its data.

* **Input**: Store name and error handling flag.

* **Behavior**: Permanently removes the store and all entries.

* **Response**: Confirmation of store deletion.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { DropStore } from "ahnlich-client-node/grpc/db/query_pb";

async function dropStore() {
  const client = createDbClient("127.0.0.1:1369");

  await client.dropStore(
    new DropStore({
      store: "my_store",
      errorIfNotExists: true,
    })
  );

  console.log("Store dropped successfully");
}

dropStore();
```
</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `store` | `string` | Yes | The name of the store to delete |
| `errorIfNotExists` | `boolean` | No | If `true`, throws error if store doesn't exist |

## Notes

- **This operation is irreversible** - all data in the store will be permanently deleted
- All indices (predicate and non-linear) are also removed
- Use `errorIfNotExists: false` for idempotent cleanup scripts
