---
title: List Connected Clients
---

# List Connected Clients

The ListClients request retrieves a list of all clients currently connected to the Ahnlich DB server.

* **Input**: No arguments required.

* **Behavior**: The server returns information about all active client connections.

* **Response**: A list of connected client information including client addresses.

<details>
  <summary>Click to expand source code</summary>

  ```py
  import asyncio
  from grpclib.client import Channel
  from ahnlich_client_py.grpc.services.db_service import DbServiceStub
  from ahnlich_client_py.grpc.db import query as db_query

  async def list_connected_clients():
      async with Channel(host="127.0.0.1", port=1369) as channel:
          client = DbServiceStub(channel)
          
          response = await client.list_clients(db_query.ListClients())
          
          # response.clients contains information about connected clients
          for client_info in response.clients:
              print(f"Connected client: {client_info.address}")

  if __name__ == "__main__":
      asyncio.run(list_connected_clients())
  ```
</details>
