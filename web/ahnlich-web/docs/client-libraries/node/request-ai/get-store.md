---
title: Get Store
sidebar_position: 4
---

# Get Store

The GetStore request retrieves detailed information about a single AI store by its name.

* **Input**: Store name.

* **Behavior**: The server returns detailed information about the specified AI store.

* **Response**: AI store information including models, dimension, and indices.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createAiClient } from "ahnlich-client-node";
import { GetStore } from "ahnlich-client-node/grpc/ai/query_pb";

async function getStore() {
  const client = createAiClient("127.0.0.1:1370");

  const response = await client.getStore(
    new GetStore({ store: "ai_store" })
  );

  console.log(response.name);            // Store name
  console.log(response.queryModel);      // AI model used for querying
  console.log(response.indexModel);      // AI model used for indexing
  console.log(response.embeddingSize);   // Number of stored embeddings
  console.log(response.dimension);       // Vector dimension
  console.log(response.predicateIndices); // Indexed predicate keys
  console.log(response.dbInfo);          // Optional DB store info
}

getStore();
```
</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `store` | `string` | Yes | The name of the AI store to retrieve |

## Response Fields

| Field | Type | Description |
|-------|------|-------------|
| `name` | `string` | The name of the store |
| `queryModel` | `AIModel` | AI model used for query embedding |
| `indexModel` | `AIModel` | AI model used for indexing |
| `embeddingSize` | `number` | Number of stored embeddings |
| `dimension` | `number` | Vector dimension |
| `predicateIndices` | `string[]` | List of indexed predicate keys |
| `dbInfo` | `StoreInfo` | Optional underlying DB store info |
