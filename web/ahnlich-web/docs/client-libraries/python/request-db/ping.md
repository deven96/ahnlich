---
title: Ping
sidebar_posiiton: 1
---

# Ping

The Ping request is used to test the connectivity between the Python client and the Ahnlich DB server. 
It acts as a health check to confirm that the DB service is up and running, and it is also useful for debugging or monitoring setups.

* **Input**: No arguments are required. You may pass optional metadata, such as tracing IDs, for observability and distributed tracing.

* **Behavior**: The client sends a simple ping message to the DB server, and the server responds with a Pong.

* **Response**: A Pong message confirming connectivity.

<details>
  <summary>Click to expand source code</summary>

```py
import asyncio
from grpclib.client import Channel
from ahnlich_client_py.grpc.services.db_service import DbServiceStub
from ahnlich_client_py.grpc.db import query as db_query
from ahnlich_client_py.grpc.db.server import Pong

async def Ping():

  """
  Test ping
  """

  # Initialize client

  async with Channel(host="127.0.0.1", port=1369) as channel:

    db_client = DbServiceStub(channel)

    # Prepare tracing metadata

    tracing_id = "00-80e1afed08e019fc1110464cfa66635c-7a085853722dc6d2-01"

    metadata = {"ahnlich-trace-id": tracing_id}

    # Make request with metadata

    response = await db_client.ping(

      db_query.Ping(),

      metadata=metadata

    )

if __name__ == "__main__":
  asyncio.run(Ping()) 
```
</details>