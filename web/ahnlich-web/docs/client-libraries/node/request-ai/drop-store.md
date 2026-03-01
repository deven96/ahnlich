---
title: Drop Store
sidebar_position: 14
---

# Drop Store

Deletes an entire AI store and all its data.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createAiClient } from "ahnlich-client-node";
import { DropStore } from "ahnlich-client-node/grpc/ai/query_pb";

async function dropStore() {
  const client = createAiClient("127.0.0.1:1370");

  await client.dropStore(
    new DropStore({
      store: "ai_store",
      errorIfNotExists: true,
    })
  );

  console.log("AI store dropped");
}

dropStore();
```
</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `store` | `string` | Yes | The name of the AI store to delete |
| `errorIfNotExists` | `boolean` | No | If `true`, throws error if store doesn't exist |

## Notes

- **This operation is irreversible**
- All data, embeddings, and indices are permanently deleted
