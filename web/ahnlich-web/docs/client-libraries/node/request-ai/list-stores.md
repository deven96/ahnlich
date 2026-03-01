---
title: List Stores
sidebar_position: 3
---

# List Stores

The ListStores request retrieves a list of all AI stores available on the Ahnlich AI server.

* **Input**: No arguments required.

* **Behavior**: The server returns information about all existing AI stores.

* **Response**: A list of AI store information.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createAiClient } from "ahnlich-client-node";
import { ListStores } from "ahnlich-client-node/grpc/ai/query_pb";

async function listStores() {
  const client = createAiClient("127.0.0.1:1370");

  const response = await client.listStores(new ListStores());

  console.log(response.stores.map((s) => s.name));

  for (const store of response.stores) {
    console.log(`Store: ${store.name}`);
    console.log(`  Query Model: ${store.queryModel}`);
    console.log(`  Index Model: ${store.indexModel}`);
    console.log(`  Embedding Size: ${store.embeddingSize}`);
  }
}

listStores();
```
</details>
