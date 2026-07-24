---
title: List Stores
sidebar_position: 3
---

# List Stores

## Schema

`ListStores` accepts an optional `schema` field. When it is omitted, the server lists stores in `public` only; it does not list stores across every schema. Set `schema` to list stores in another schema.

The ListStores request retrieves AI stores from one schema on the Ahnlich AI server. When `schema` is omitted, that schema is `public`.

* **Input**: Optional `schema` field.

* **Behavior**: The server returns AI stores in the requested schema.

* **Response**: A list of AI store information.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createAiClient } from "ahnlich-client-node";
import { ListStores } from "ahnlich-client-node/grpc/ai/query_pb";

async function listStores() {
  const client = createAiClient("127.0.0.1:1370");

  const response = await client.listStores(new ListStores({ schema: "analytics" }));

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
