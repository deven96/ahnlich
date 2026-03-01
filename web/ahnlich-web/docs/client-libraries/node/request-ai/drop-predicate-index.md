---
title: Drop Predicate Index
sidebar_position: 10
---

# Drop Predicate Index

Removes an existing predicate index from an AI store.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createAiClient } from "ahnlich-client-node";
import { DropPredIndex } from "ahnlich-client-node/grpc/ai/query_pb";

async function dropPredicateIndex() {
  const client = createAiClient("127.0.0.1:1370");

  await client.dropPredIndex(
    new DropPredIndex({
      store: "ai_store",
      predicates: ["brand"],
      errorIfNotExists: true,
    })
  );

  console.log("Predicate index dropped");
}

dropPredicateIndex();
```
</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `store` | `string` | Yes | The name of the AI store |
| `predicates` | `string[]` | Yes | List of predicate indices to remove |
| `errorIfNotExists` | `boolean` | No | If `true`, throws error if index doesn't exist |
