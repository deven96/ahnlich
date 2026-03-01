---
title: Node.js Specific Resources
sidebar_position: 1
---

## QuickStart

Set up your local development environment to start building with Ahnlich using Node.js/TypeScript.

## Install Node.js

Go to the official [Node.js website](https://nodejs.org/) and download the LTS version.

Ensure that Node.js is installed on your system, and verify its version by running the following command in your terminal:

```bash
node -v
```

*Example output:*

```
v20.11.0
```

Also verify npm is available:

```bash
npm -v
```

## Set Up a Node.js Project

Create a new directory for your project and initialize it:

```bash
mkdir ahnlich-app
cd ahnlich-app
npm init -y
```

If you're using TypeScript (recommended), install TypeScript and initialize it:

```bash
npm install typescript @types/node --save-dev
npx tsc --init
```

## Install the Ahnlich Client Node

Install the Ahnlich Node.js client SDK using npm:

```bash
npm install ahnlich-client-node
```

Or using yarn:

```bash
yarn add ahnlich-client-node
```

Or using pnpm:

```bash
pnpm add ahnlich-client-node
```

## Package Information

The Ahnlich Node.js client provides a gRPC-based SDK for interacting with Ahnlich-DB (vector storage, similarity search) and Ahnlich-AI (semantic models).

### Features

* gRPC service clients for DB and AI via [`@connectrpc/connect`](https://connectrpc.com/)

* TypeScript types generated from the Ahnlich `.proto` definitions

* Optional auth (TLS + bearer token) and trace ID support

### Modules

* `ahnlich-client-node` - Main package export with `createDbClient` and `createAiClient`

* `ahnlich-client-node/grpc/db/query_pb` - DB query types (Ping, CreateStore, Set, etc.)

* `ahnlich-client-node/grpc/ai/query_pb` - AI query types

* `ahnlich-client-node/grpc/keyval_pb` - Key-value types (StoreKey, StoreValue, StoreInput, etc.)

* `ahnlich-client-node/grpc/metadata_pb` - Metadata types (MetadataValue)

* `ahnlich-client-node/grpc/predicate_pb` - Predicate types (PredicateCondition, Equals, etc.)

* `ahnlich-client-node/grpc/algorithm/algorithm_pb` - Algorithm types (Algorithm enum)

* `ahnlich-client-node/grpc/algorithm/nonlinear_pb` - Non-linear algorithm types (KDTreeConfig, HNSWConfig)

* `ahnlich-client-node/grpc/ai/models_pb` - AI model types (AIModel enum)

### Initialization

Every request starts by creating a client connection to the Ahnlich server.

#### DB Client

```ts
import { createDbClient } from "ahnlich-client-node";

const client = createDbClient("127.0.0.1:1369");

// Client is now ready for requests
const response = await client.ping(new Ping());
```

#### AI Client

```ts
import { createAiClient } from "ahnlich-client-node";

const client = createAiClient("127.0.0.1:1370");

// Client is now ready for requests
const response = await client.ping(new Ping());
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

### With Tracing

Pass a trace ID to correlate requests across services:

```ts
import { createDbClient } from "ahnlich-client-node";

const client = createDbClient("127.0.0.1:1369", {
  traceId: "00-80e1afed08e019fc1110464cfa66635c-7a085853722dc6d2-01",
});
```

This sets the `ahnlich-trace-id` header on every request.
