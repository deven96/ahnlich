---
title: Info Server
sidebar_position: 2
---

# Info Server

The InfoServer request retrieves metadata about the Ahnlich AI server.

* **Input**: No arguments required.

* **Behavior**: The client requests server information.

* **Response**: Server information including version and supported models.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createAiClient } from "ahnlich-client-node";
import { InfoServer } from "ahnlich-client-node/grpc/ai/query_pb";

async function infoServer() {
  const client = createAiClient("127.0.0.1:1370");

  const response = await client.infoServer(new InfoServer());

  console.log(response.info?.version);
}

infoServer();
```
</details>
