---
title: Info Server
sidebar_position: 2
---

# Info Server

The InfoServer request retrieves metadata about the Ahnlich DB server, including version information and other server details.

* **Input**: No arguments required.

* **Behavior**: The client requests server information, and the server responds with its metadata.

* **Response**: Server information including version.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { InfoServer } from "ahnlich-client-node/grpc/db/query_pb";

async function infoServer() {
  const client = createDbClient("127.0.0.1:1369");

  const response = await client.infoServer(new InfoServer());

  console.log(response.info?.version);    // Server version
  console.log(response.info?.address);    // Server address
  console.log(response.info?.numStores);  // Number of stores
}

infoServer();
```
</details>
