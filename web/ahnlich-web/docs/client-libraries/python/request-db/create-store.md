---
title: Create Store
---

# Create Store

A Store in Ahnlich is like a logical dataset or collection. Each store holds vectors and their associated metadata, allowing you to organize data by application, environment, or project.

* **Behavior**: Creates a new isolated vector store. Multiple stores can coexist, enabling different workloads.

* **Parameters**:
  - `store`: Unique name for the store
  - `dimension`: Vector dimensionality (all vectors must match this)
  - `create_predicates`: List of metadata field names to enable filtering (can be empty)
  - `non_linear_indices`: List of non-linear algorithms for approximate search (can be empty)
  - `error_if_exists`: If True, returns error when store already exists

* **Response**: A confirmation message (Unit).

<details>
  <summary>Click to expand source code</summary>

```py
import asyncio
from grpclib.client import Channel
from ahnlich_client_py.grpc.services.db_service import DbServiceStub
from ahnlich_client_py.grpc.db import query as db_query
from ahnlich_client_py.grpc.db.server import Unit

async def create_store():
  async with Channel(host="127.0.0.1", port=1369) as channel:
    client = DbServiceStub(channel)
    response = await client.create_store(
      db_query.CreateStore(
        store="test store 006",
        dimension=5,  # Fixed vector dimension
        create_predicates=["job"],  # Index these metadata fields
        non_linear_indices=[],  # Optional: non-linear algorithms for faster search
        error_if_exists=True
      )
    )
    # response is Unit() on success
  
    # All store_keys must match this dimension
    # Example valid key:
    valid_key = [1.0, 2.0, 3.0, 4.0, 5.0]  # length = 5
  assert isinstance(response, Unit)
if __name__ == "__main__":
  asyncio.run(create_store())
```
</details>