---
title: Ping
sidebar_position: 1
---

# Ping

The Ping request is used to test the connectivity between the Node.js client and the Ahnlich DB server. 
It acts as a health check to confirm that the DB service is up and running, and it is also useful for debugging or monitoring setups.

* **Input**: No arguments are required. You may pass optional metadata, such as tracing IDs, for observability and distributed tracing.

* **Behavior**: The client sends a simple ping message to the DB server, and the server responds with a Pong.

* **Response**: A Pong message confirming connectivity.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { Ping } from "ahnlich-client-node/grpc/db/query_pb";

async function ping() {
  // Initialize client
  const client = createDbClient("127.0.0.1:1369");

  // Make request
  const response = await client.ping(new Ping());

  console.log(response); // Pong
}

ping();
```
</details>

### With Tracing

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { Ping } from "ahnlich-client-node/grpc/db/query_pb";

async function pingWithTracing() {
  // Initialize client with trace ID
  const client = createDbClient("127.0.0.1:1369", {
    traceId: "00-80e1afed08e019fc1110464cfa66635c-7a085853722dc6d2-01",
  });

  // Make request - trace ID is automatically included
  const response = await client.ping(new Ping());

  console.log(response); // Pong
}

pingWithTracing();
```
</details>
