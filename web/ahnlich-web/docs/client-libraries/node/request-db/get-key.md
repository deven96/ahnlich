---
title: Get Key
sidebar_position: 9
---

# Get Key

The GetKey request retrieves entries from a store by their exact vector keys.

* **Input**: Store name and array of keys to retrieve.

* **Behavior**: Returns entries that exactly match the provided keys.

* **Response**: A list of matching entries with their values.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { GetKey } from "ahnlich-client-node/grpc/db/query_pb";
import { StoreKey } from "ahnlich-client-node/grpc/keyval_pb";

async function getKey() {
  const client = createDbClient("127.0.0.1:1369");

  const response = await client.getKey(
    new GetKey({
      store: "my_store",
      keys: [new StoreKey({ key: [1.0, 2.0, 3.0, 4.0] })],
    })
  );

  console.log(response.entries);

  // Iterate over results
  for (const entry of response.entries) {
    console.log(`Key: ${entry.key?.key}`);
    console.log(`Value: ${JSON.stringify(entry.value?.value)}`);
  }
}

getKey();
```
</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `store` | `string` | Yes | The name of the store |
| `keys` | `StoreKey[]` | Yes | Array of vector keys to retrieve |

## Example with Multiple Keys

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { GetKey } from "ahnlich-client-node/grpc/db/query_pb";
import { StoreKey } from "ahnlich-client-node/grpc/keyval_pb";

async function getMultipleKeys() {
  const client = createDbClient("127.0.0.1:1369");

  const response = await client.getKey(
    new GetKey({
      store: "my_store",
      keys: [
        new StoreKey({ key: [1.0, 2.0, 3.0, 4.0] }),
        new StoreKey({ key: [5.0, 6.0, 7.0, 8.0] }),
        new StoreKey({ key: [9.0, 10.0, 11.0, 12.0] }),
      ],
    })
  );

  console.log(`Retrieved ${response.entries.length} entries`);
}

getMultipleKeys();
```
</details>

## Notes

- Keys must exactly match stored vectors (including floating-point precision)
- If a key is not found, it will simply not appear in the results
- For similarity-based retrieval, use [GetSimN](/docs/client-libraries/node/request-db/get-simn) instead
