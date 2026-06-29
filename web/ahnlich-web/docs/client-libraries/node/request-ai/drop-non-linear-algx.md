---
title: Drop Non Linear Algorithm Index
sidebar_position: 12
---

# Drop Non Linear Algorithm Index

## Schema

This request accepts an optional `schema` field. When it is omitted, the server uses the `public` schema. Set `schema` to target a store in another schema.

Removes a non-linear index (KDTree or HNSW) from an AI store.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createAiClient } from "ahnlich-client-node";
import { DropNonLinearAlgorithmIndex } from "ahnlich-client-node/grpc/ai/query_pb";
import { NonLinearAlgorithm } from "ahnlich-client-node/grpc/algorithm/nonlinear_pb";

async function dropNonLinearIndex() {
  const client = createAiClient("127.0.0.1:1370");

  await client.dropNonLinearAlgorithmIndex(
    new DropNonLinearAlgorithmIndex({
      store: "ai_store",
      schema: "analytics",
      nonLinearIndices: [NonLinearAlgorithm.KDTree],
      errorIfNotExists: true,
    })
  );

  console.log("KDTree index dropped");
}

dropNonLinearIndex();
```
</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `store` | `string` | Yes | The name of the AI store |
| `nonLinearIndices` | `NonLinearAlgorithm[]` | Yes | List of index types to remove |
| `errorIfNotExists` | `boolean` | No | If `true`, throws error if index doesn't exist |
