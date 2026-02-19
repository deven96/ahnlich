# Ahnlich Node.js Client SDK

[![Ahnlich TestSuite](https://github.com/deven96/ahnlich/actions/workflows/test.yml/badge.svg)](https://github.com/deven96/ahnlich/actions/workflows/test.yml)
[![Node Client Tag and Deploy](https://github.com/deven96/ahnlich/actions/workflows/node_tag_and_deploy.yml/badge.svg)](https://github.com/deven96/ahnlich/actions/workflows/node_tag_and_deploy.yml)

A Node.js/TypeScript client that interacts with both Ahnlich DB and AI over gRPC.

## Table of Contents

- [Installation](#installation)
- [Package Information](#package-information)
- [Initialization](#initialization)
  - [DB Client](#db-client)
  - [AI Client](#ai-client)
  - [With Authentication](#with-authentication)
- [Requests - DB](#requests---db)
  - [Ping](#ping)
  - [Info Server](#info-server)
  - [List Connected Clients](#list-connected-clients)
  - [List Stores](#list-stores)
  - [Create Store](#create-store)
  - [Set](#set)
  - [Get Sim N](#get-sim-n)
  - [Get Key](#get-key)
  - [Get By Predicate](#get-by-predicate)
  - [Create Predicate Index](#create-predicate-index)
  - [Drop Predicate Index](#drop-predicate-index)
  - [Create Non Linear Algorithm Index](#create-non-linear-algorithm-index)
  - [Drop Non Linear Algorithm Index](#drop-non-linear-algorithm-index)
  - [Delete Key](#delete-key)
  - [Delete Predicate](#delete-predicate)
  - [Drop Store](#drop-store)
- [Requests - AI](#requests---ai)
  - [Ping](#ping-1)
  - [Info Server](#info-server-1)
  - [List Stores](#list-stores-1)
  - [Create Store](#create-store-1)
  - [Set](#set-1)
  - [Get Sim N](#get-sim-n-1)
  - [Get By Predicate](#get-by-predicate-1)
  - [Create Predicate Index](#create-predicate-index-1)
  - [Drop Predicate Index](#drop-predicate-index-1)
  - [Create Non Linear Algorithm Index](#create-non-linear-algorithm-index-1)
  - [Drop Non Linear Algorithm Index](#drop-non-linear-algorithm-index-1)
  - [Delete Key](#delete-key-1)
  - [Drop Store](#drop-store-1)
- [Tracing](#tracing)
- [Development & Testing](#development--testing)
- [Deploy to npm](#deploy-to-npm)
- [Type Meanings](#type-meanings)
- [Change Log](#change-log)

## Installation

```bash
npm install ahnlich-client-node
```

## Package Information

This package provides:
- gRPC service clients for DB and AI via [`@connectrpc/connect`](https://connectrpc.com/)
- TypeScript types generated from the Ahnlich `.proto` definitions
- Optional auth (TLS + bearer token) and trace ID support

## Initialization

### DB Client

```ts
import { createDbClient } from "ahnlich-client-node";

const client = createDbClient("127.0.0.1:1369");
```

### AI Client

```ts
import { createAiClient } from "ahnlich-client-node";

const client = createAiClient("127.0.0.1:1370");
```

### With Authentication

When the server is started with `--enable-auth`, pass a CA certificate and credentials:

```ts
import * as fs from "fs";
import { createDbClient } from "ahnlich-client-node";

const client = createDbClient("127.0.0.1:1369", {
  caCert: fs.readFileSync("ca.crt"),
  auth: { username: "myuser", apiKey: "mykey" },
});
```

Pass a trace ID to correlate requests across services:

```ts
const client = createDbClient("127.0.0.1:1369", {
  traceId: "00-80e1afed08e019fc1110464cfa66635c-7a085853722dc6d2-01",
});
```

---

## Requests - DB

### Ping

```ts
import { Ping } from "ahnlich-client-node/grpc/db/query_pb";

const response = await client.ping(new Ping());
console.log(response); // Pong
```

### Info Server

```ts
import { InfoServer } from "ahnlich-client-node/grpc/db/query_pb";

const response = await client.infoServer(new InfoServer());
console.log(response.info?.version);
```

### List Connected Clients

```ts
import { ListClients } from "ahnlich-client-node/grpc/db/query_pb";

const response = await client.listClients(new ListClients());
console.log(response.clients);
```

### List Stores

```ts
import { ListStores } from "ahnlich-client-node/grpc/db/query_pb";

const response = await client.listStores(new ListStores());
console.log(response.stores.map((s) => s.name));
```

### Create Store

```ts
import { CreateStore } from "ahnlich-client-node/grpc/db/query_pb";

await client.createStore(
  new CreateStore({
    store: "my_store",
    dimension: 4,
    predicates: ["label"],
    errorIfExists: true,
  }),
);
```

Store dimension is fixed at creation — all inserted vectors must match it.

### Set

```ts
import { Set } from "ahnlich-client-node/grpc/db/query_pb";
import { DbStoreEntry, StoreKey, StoreValue } from "ahnlich-client-node/grpc/keyval_pb";
import { MetadataValue } from "ahnlich-client-node/grpc/metadata_pb";

await client.set(
  new Set({
    store: "my_store",
    inputs: [
      new DbStoreEntry({
        key: new StoreKey({ key: [1.0, 2.0, 3.0, 4.0] }),
        value: new StoreValue({
          value: {
            label: new MetadataValue({ value: { case: "rawString", value: "A" } }),
          },
        }),
      }),
    ],
  }),
);
```

### Get Sim N

Returns the closest N entries to a query vector.

```ts
import { GetSimN } from "ahnlich-client-node/grpc/db/query_pb";
import { StoreKey } from "ahnlich-client-node/grpc/keyval_pb";
import { Algorithm } from "ahnlich-client-node/grpc/algorithm/algorithm_pb";

const response = await client.getSimN(
  new GetSimN({
    store: "my_store",
    searchInput: new StoreKey({ key: [1.0, 2.0, 3.0, 4.0] }),
    closestN: 3,
    algorithm: Algorithm.COSINE_SIMILARITY,
  }),
);
console.log(response.entries);
```

### Get Key

```ts
import { GetKey } from "ahnlich-client-node/grpc/db/query_pb";
import { StoreKey } from "ahnlich-client-node/grpc/keyval_pb";

const response = await client.getKey(
  new GetKey({
    store: "my_store",
    keys: [new StoreKey({ key: [1.0, 2.0, 3.0, 4.0] })],
  }),
);
console.log(response.entries);
```

### Get By Predicate

```ts
import { GetPred } from "ahnlich-client-node/grpc/db/query_pb";
import { PredicateCondition, Predicate, Equals } from "ahnlich-client-node/grpc/predicate_pb";
import { MetadataValue } from "ahnlich-client-node/grpc/metadata_pb";

const response = await client.getPred(
  new GetPred({
    store: "my_store",
    condition: new PredicateCondition({
      kind: {
        case: "value",
        value: new Predicate({
          kind: {
            case: "equals",
            value: new Equals({
              key: "label",
              value: new MetadataValue({ value: { case: "rawString", value: "A" } }),
            }),
          },
        }),
      },
    }),
  }),
);
```

### Create Predicate Index

```ts
import { CreatePredIndex } from "ahnlich-client-node/grpc/db/query_pb";

await client.createPredIndex(
  new CreatePredIndex({ store: "my_store", predicates: ["label"] }),
);
```

### Drop Predicate Index

```ts
import { DropPredIndex } from "ahnlich-client-node/grpc/db/query_pb";

await client.dropPredIndex(
  new DropPredIndex({ store: "my_store", predicates: ["label"], errorIfNotExists: true }),
);
```

### Create Non Linear Algorithm Index

```ts
import { CreateNonLinearAlgorithmIndex } from "ahnlich-client-node/grpc/db/query_pb";
import { NonLinearAlgorithm } from "ahnlich-client-node/grpc/algorithm/nonlinear_pb";

await client.createNonLinearAlgorithmIndex(
  new CreateNonLinearAlgorithmIndex({
    store: "my_store",
    nonLinearIndices: [NonLinearAlgorithm.KD_TREE],
  }),
);
```

### Drop Non Linear Algorithm Index

```ts
import { DropNonLinearAlgorithmIndex } from "ahnlich-client-node/grpc/db/query_pb";
import { NonLinearAlgorithm } from "ahnlich-client-node/grpc/algorithm/nonlinear_pb";

await client.dropNonLinearAlgorithmIndex(
  new DropNonLinearAlgorithmIndex({
    store: "my_store",
    nonLinearIndices: [NonLinearAlgorithm.KD_TREE],
    errorIfNotExists: true,
  }),
);
```

### Delete Key

```ts
import { DelKey } from "ahnlich-client-node/grpc/db/query_pb";
import { StoreKey } from "ahnlich-client-node/grpc/keyval_pb";

await client.delKey(
  new DelKey({
    store: "my_store",
    keys: [new StoreKey({ key: [1.0, 2.0, 3.0, 4.0] })],
  }),
);
```

### Delete Predicate

```ts
import { DelPred } from "ahnlich-client-node/grpc/db/query_pb";

await client.delPred(
  new DelPred({
    store: "my_store",
    condition: /* same PredicateCondition as Get By Predicate */,
  }),
);
```

### Drop Store

```ts
import { DropStore } from "ahnlich-client-node/grpc/db/query_pb";

await client.dropStore(new DropStore({ store: "my_store", errorIfNotExists: true }));
```

---

## Requests - AI

### Ping

```ts
import { Ping } from "ahnlich-client-node/grpc/ai/query_pb";

const response = await client.ping(new Ping());
```

### Info Server

```ts
import { InfoServer } from "ahnlich-client-node/grpc/ai/query_pb";

const response = await client.infoServer(new InfoServer());
```

### List Stores

```ts
import { ListStores } from "ahnlich-client-node/grpc/ai/query_pb";

const response = await client.listStores(new ListStores());
console.log(response.stores.map((s) => s.name));
```

### Create Store

```ts
import { CreateStore } from "ahnlich-client-node/grpc/ai/query_pb";
import { AIModel } from "ahnlich-client-node/grpc/ai/models_pb";

await client.createStore(
  new CreateStore({
    store: "ai_store",
    queryModel: AIModel.ALL_MINI_LM_L6_V2,
    indexModel: AIModel.ALL_MINI_LM_L6_V2,
    predicates: ["brand"],
    errorIfExists: true,
    storeOriginal: true,
  }),
);
```

### Set

```ts
import { Set } from "ahnlich-client-node/grpc/ai/query_pb";
import { AiStoreEntry, StoreInput, StoreValue } from "ahnlich-client-node/grpc/keyval_pb";
import { MetadataValue } from "ahnlich-client-node/grpc/metadata_pb";
import { PreprocessAction } from "ahnlich-client-node/grpc/ai/preprocess_pb";

await client.set(
  new Set({
    store: "ai_store",
    inputs: [
      new AiStoreEntry({
        key: new StoreInput({ value: { case: "rawString", value: "Jordan One" } }),
        value: new StoreValue({
          value: {
            brand: new MetadataValue({ value: { case: "rawString", value: "Nike" } }),
          },
        }),
      }),
    ],
    preprocessAction: PreprocessAction.NO_PREPROCESSING,
  }),
);
```

### Get Sim N

```ts
import { GetSimN } from "ahnlich-client-node/grpc/ai/query_pb";
import { StoreInput } from "ahnlich-client-node/grpc/keyval_pb";
import { Algorithm } from "ahnlich-client-node/grpc/algorithm/algorithm_pb";

const response = await client.getSimN(
  new GetSimN({
    store: "ai_store",
    searchInput: new StoreInput({ value: { case: "rawString", value: "Jordan" } }),
    closestN: 3,
    algorithm: Algorithm.COSINE_SIMILARITY,
  }),
);
console.log(response.entries);
```

### Get By Predicate

```ts
import { GetPred } from "ahnlich-client-node/grpc/ai/query_pb";

const response = await client.getPred(
  new GetPred({
    store: "ai_store",
    condition: /* PredicateCondition — same structure as DB */,
  }),
);
```

### Create Predicate Index

```ts
import { CreatePredIndex } from "ahnlich-client-node/grpc/ai/query_pb";

await client.createPredIndex(
  new CreatePredIndex({ store: "ai_store", predicates: ["brand"] }),
);
```

### Drop Predicate Index

```ts
import { DropPredIndex } from "ahnlich-client-node/grpc/ai/query_pb";

await client.dropPredIndex(
  new DropPredIndex({ store: "ai_store", predicates: ["brand"], errorIfNotExists: true }),
);
```

### Create Non Linear Algorithm Index

```ts
import { CreateNonLinearAlgorithmIndex } from "ahnlich-client-node/grpc/ai/query_pb";
import { NonLinearAlgorithm } from "ahnlich-client-node/grpc/algorithm/nonlinear_pb";

await client.createNonLinearAlgorithmIndex(
  new CreateNonLinearAlgorithmIndex({
    store: "ai_store",
    nonLinearIndices: [NonLinearAlgorithm.KD_TREE],
  }),
);
```

### Drop Non Linear Algorithm Index

```ts
import { DropNonLinearAlgorithmIndex } from "ahnlich-client-node/grpc/ai/query_pb";
import { NonLinearAlgorithm } from "ahnlich-client-node/grpc/algorithm/nonlinear_pb";

await client.dropNonLinearAlgorithmIndex(
  new DropNonLinearAlgorithmIndex({
    store: "ai_store",
    nonLinearIndices: [NonLinearAlgorithm.KD_TREE],
    errorIfNotExists: true,
  }),
);
```

### Delete Key

```ts
import { DelKey } from "ahnlich-client-node/grpc/ai/query_pb";
import { StoreInput } from "ahnlich-client-node/grpc/keyval_pb";

await client.delKey(
  new DelKey({
    store: "ai_store",
    keys: [new StoreInput({ value: { case: "rawString", value: "Jordan One" } })],
  }),
);
```

### Drop Store

```ts
import { DropStore } from "ahnlich-client-node/grpc/ai/query_pb";

await client.dropStore(new DropStore({ store: "ai_store", errorIfNotExists: true }));
```

---

## Tracing

Pass a W3C trace ID to correlate requests across services:

```ts
const client = createDbClient("127.0.0.1:1369", {
  traceId: "00-80e1afed08e019fc1110464cfa66635c-7a085853722dc6d2-01",
});
```

This sets the `ahnlich-trace-id` header on every request.

---

## Development & Testing

```bash
make install-dependencies
make format
make lint-check
make test
```

## Deploy to npm

Bump the `version` field in `package.json`. When your PR is merged into `main`, the CI will detect the version change and automatically publish to npm.

## Type Meanings

- **StoreKey**: A one-dimensional `float32` vector of fixed dimension
- **StoreValue**: A map of string keys to `MetadataValue` (text or binary)
- **StoreInput**: A raw string or binary blob accepted by the AI proxy
- **Predicates**: Filter conditions for stored values (`Equals`, `NotEquals`, `In`, etc.)
- **PredicateCondition**: Combines one or more predicates with `AND`, `OR`, or `Value`
- **AIModel**: Supported embedding models (`ALL_MINI_LM_L6_V2`, `RESNET50`, `BUFFALO_L`, etc.)

## Change Log

| Version | Description                                      |
|---------|--------------------------------------------------|
| 0.1.0   | Initial Node.js/TypeScript SDK release via gRPC  |
