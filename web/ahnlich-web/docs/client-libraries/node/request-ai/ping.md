---
title: Ping
sidebar_position: 1
---

# Ping

The Ping request is used to test the connectivity between the Node.js client and the Ahnlich AI server.

* **Input**: No arguments required.

* **Behavior**: The client sends a ping message to the AI server, and the server responds with a Pong.

* **Response**: A Pong message confirming connectivity.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createAiClient } from "ahnlich-client-node";
import { Ping } from "ahnlich-client-node/grpc/ai/query_pb";

async function ping() {
  const client = createAiClient("127.0.0.1:1370");

  const response = await client.ping(new Ping());

  console.log(response); // Pong
}

ping();
```
</details>
