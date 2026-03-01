---
title: Delete Key
sidebar_position: 15
---

# Delete Key

The DelKey request deletes entries from a store by their exact vector keys.

* **Input**: Store name and array of keys to delete.

* **Behavior**: Removes entries that exactly match the provided keys.

* **Response**: Confirmation of deletion.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { DelKey } from "ahnlich-client-node/grpc/db/query_pb";
import { StoreKey } from "ahnlich-client-node/grpc/keyval_pb";

async function deleteKey() {
  const client = createDbClient("127.0.0.1:1369");

  await client.delKey(
    new DelKey({
      store: "my_store",
      keys: [new StoreKey({ key: [1.0, 2.0, 3.0, 4.0] })],
    })
  );

  console.log("Entry deleted successfully");
}

deleteKey();
```
</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `store` | `string` | Yes | The name of the store |
| `keys` | `StoreKey[]` | Yes | Array of vector keys to delete |

## Example with Multiple Keys

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { DelKey } from "ahnlich-client-node/grpc/db/query_pb";
import { StoreKey } from "ahnlich-client-node/grpc/keyval_pb";

async function deleteMultipleKeys() {
  const client = createDbClient("127.0.0.1:1369");

  await client.delKey(
    new DelKey({
      store: "my_store",
      keys: [
        new StoreKey({ key: [1.0, 2.0, 3.0, 4.0] }),
        new StoreKey({ key: [5.0, 6.0, 7.0, 8.0] }),
        new StoreKey({ key: [9.0, 10.0, 11.0, 12.0] }),
      ],
    })
  );

  console.log("Entries deleted successfully");
}

deleteMultipleKeys();
```
</details>

## Notes

- Keys must exactly match stored vectors
- Non-existent keys are silently ignored
- For bulk deletion by metadata, use [Delete Predicate](/docs/client-libraries/node/request-db/delete-predicate)
