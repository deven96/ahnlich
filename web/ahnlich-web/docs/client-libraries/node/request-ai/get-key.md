---
title: GetKey
---

# GetKey

## Schema

This request accepts an optional `schema` field. When it is omitted, the server uses the `public` schema. Set `schema` to target a store in another schema.

The GetKey request retrieves entries from a store based on exact input matches.

* **Input**:
  * `store`: the store name.
  * `keys`: the exact inputs (text, image, or audio) you want to retrieve.

* **Behavior**: Finds the stored entries that match the inputs exactly.

* **Response**: Returns the entries (input + metadata) if found.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createAiClient } from "ahnlich-client-node";
import { GetKey } from "ahnlich-client-node/grpc/ai/query_pb";
import { StoreInput } from "ahnlich-client-node/grpc/keyval_pb";

async function getKey() {
  const client = createAiClient("127.0.0.1:1370");

  // Create the input to look up (using oneof discriminated union)
  const storeInput = new StoreInput({
    value: {
      case: "rawString",
      value: "Your search text"
    }
  });

  const response = await client.getKey(
    new GetKey({
      store: "my_store",
      schema: "analytics",
      keys: [storeInput]
    })
  );

  // response.entries contains matching (key, value) pairs
  for (const entry of response.entries) {
    console.log("Found entry:", entry);
  }
}

getKey();
```
</details>
