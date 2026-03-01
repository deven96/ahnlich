---
title: List Stores
---

# List Stores

The **List Stores** request retrieves the set of Stores currently registered on the DB server. Each Store corresponds to a logical container of vectors. This operation is commonly used for **introspection**, **administrative tooling**, and **debugging**.

## Behavior
* The client sends a ListStores request.
* The server responds with a collection of registered Stores. Each store entry includes: name, entry count, size in bytes, and non-linear index configurations (HNSW parameters or k-d tree) if any are active.
* An empty list means no Stores have been created yet.

<details>
  <summary>Click to expand source code</summary>

```py
import asyncio
from grpclib.client import Channel
from ahnlich_client_py.grpc.services.db_service import DbServiceStub
from ahnlich_client_py.grpc.db import query as db_query

async def list_stores():
  async with Channel(host="127.0.0.1", port=1369) as channel:
    client = DbServiceStub(channel)
  tracing_id = "00-80e1afed08e019fc1110464cfa66635c-7a085853722dc6d2-01"
  response = await client.list_stores(
    db_query.ListStores(),
    metadata={"ahnlich-trace-id": tracing_id}
  )
  print(f"Stores: {[store.name for store in response.stores]}")
  
if __name__ == "__main__":
  asyncio.run(list_stores())   
``` 
</details>