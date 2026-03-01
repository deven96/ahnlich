---
title: Create Predicate Index
sidebar_position: 9
---

# Create Predicate Index

Creates an index on metadata fields to optimize predicate-based queries in an AI store.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createAiClient } from "ahnlich-client-node";
import { CreatePredIndex } from "ahnlich-client-node/grpc/ai/query_pb";

async function createPredicateIndex() {
  const client = createAiClient("127.0.0.1:1370");

  await client.createPredIndex(
    new CreatePredIndex({
      store: "ai_store",
      predicates: ["brand", "category"],
    })
  );

  console.log("Predicate indices created");
}

createPredicateIndex();
```
</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `store` | `string` | Yes | The name of the AI store |
| `predicates` | `string[]` | Yes | List of metadata keys to index |
