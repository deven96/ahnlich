---
title: Delete Key
sidebar_position: 13
---

# Delete Key

Deletes entries from an AI store by their original input.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createAiClient } from "ahnlich-client-node";
import { DelKey } from "ahnlich-client-node/grpc/ai/query_pb";
import { StoreInput } from "ahnlich-client-node/grpc/keyval_pb";

async function deleteKey() {
  const client = createAiClient("127.0.0.1:1370");

  await client.delKey(
    new DelKey({
      store: "ai_store",
      keys: [
        new StoreInput({ value: { case: "rawString", value: "Jordan One" } }),
      ],
    })
  );

  console.log("Entry deleted");
}

deleteKey();
```
</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `store` | `string` | Yes | The name of the AI store |
| `keys` | `StoreInput[]` | Yes | Array of inputs to delete |

## Notes

- For AI stores, keys are the original inputs (text or binary), not vectors
- Non-existent keys are silently ignored
