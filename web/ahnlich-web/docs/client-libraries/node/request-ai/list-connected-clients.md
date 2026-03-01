---
title: List Connected Clients
---

# List Connected Clients

The ListClients request retrieves a list of all clients currently connected to the Ahnlich AI server.

* **Input**: No arguments required.

* **Behavior**: The server returns information about all active client connections.

* **Response**: A list of connected client information.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createAiClient } from "ahnlich-client-node";
import { ListClients } from "ahnlich-client-node/grpc/ai/query_pb";

async function listConnectedClients() {
  const client = createAiClient("127.0.0.1:1370");

  const response = await client.listClients(new ListClients());

  console.log(response.clients);

  // Iterate over connected clients
  for (const clientInfo of response.clients) {
    console.log(`Client: ${clientInfo.address}`);
  }
}

listConnectedClients();
```
</details>
