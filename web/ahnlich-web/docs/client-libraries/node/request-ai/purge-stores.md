---
title: Purge Stores
---

# Purge Stores

Deletes **all vector stores** managed by the AI server, including all embeddings and associated metadata. This is a destructive operation that resets the AI service state, typically used during testing, cleanup, or when starting fresh with new datasets.

* **Input**: No arguments required.

* **Behavior**: Removes all stores and their contents from the AI server.

* **Response**: Confirmation of deletion with count of deleted stores.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createAiClient } from "ahnlich-client-node";
import { PurgeStores } from "ahnlich-client-node/grpc/ai/query_pb";

async function purgeStores() {
  const client = createAiClient("127.0.0.1:1370");

  const response = await client.purgeStores(new PurgeStores());

  console.log(`Purged stores. Deleted count: ${response.deletedCount}`);
}

purgeStores();
```
</details>

:::warning
This operation is **irreversible**. All stores and their data will be permanently deleted. Use with caution in production environments.
:::
