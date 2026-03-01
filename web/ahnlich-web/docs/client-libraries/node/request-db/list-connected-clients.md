---
title: List Connected Clients
sidebar_position: 3
---

# List Connected Clients

The ListClients request retrieves a list of all clients currently connected to the Ahnlich DB server.

* **Input**: No arguments required.

* **Behavior**: The server returns information about all active client connections.

* **Response**: A list of connected client information.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { ListClients } from "ahnlich-client-node/grpc/db/query_pb";

async function listConnectedClients() {
  const client = createDbClient("127.0.0.1:1369");

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
