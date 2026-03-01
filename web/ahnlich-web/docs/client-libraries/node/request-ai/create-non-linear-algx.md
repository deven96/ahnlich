---
title: Create Non Linear Algorithm Index
sidebar_position: 11
---

# Create Non Linear Algorithm Index

Creates a non-linear index (KDTree or HNSW) to speed up similarity searches in an AI store.

## Create a KDTree Index

<details>
  <summary>Click to expand source code</summary>

```ts
import { createAiClient } from "ahnlich-client-node";
import { CreateNonLinearAlgorithmIndex } from "ahnlich-client-node/grpc/ai/query_pb";
import { NonLinearIndex, KDTreeConfig } from "ahnlich-client-node/grpc/algorithm/nonlinear_pb";

async function createKDTreeIndex() {
  const client = createAiClient("127.0.0.1:1370");

  await client.createNonLinearAlgorithmIndex(
    new CreateNonLinearAlgorithmIndex({
      store: "ai_store",
      nonLinearIndices: [
        new NonLinearIndex({
          index: { case: "kdtree", value: new KDTreeConfig() },
        }),
      ],
    })
  );

  console.log("KDTree index created");
}

createKDTreeIndex();
```
</details>

## Create an HNSW Index

<details>
  <summary>Click to expand source code</summary>

```ts
import { createAiClient } from "ahnlich-client-node";
import { CreateNonLinearAlgorithmIndex } from "ahnlich-client-node/grpc/ai/query_pb";
import { NonLinearIndex, HNSWConfig } from "ahnlich-client-node/grpc/algorithm/nonlinear_pb";

async function createHNSWIndex() {
  const client = createAiClient("127.0.0.1:1370");

  await client.createNonLinearAlgorithmIndex(
    new CreateNonLinearAlgorithmIndex({
      store: "ai_store",
      nonLinearIndices: [
        new NonLinearIndex({
          index: { case: "hnsw", value: new HNSWConfig() },
        }),
      ],
    })
  );

  console.log("HNSW index created");
}

createHNSWIndex();
```
</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `store` | `string` | Yes | The name of the AI store |
| `nonLinearIndices` | `NonLinearIndex[]` | Yes | List of indices to create |
