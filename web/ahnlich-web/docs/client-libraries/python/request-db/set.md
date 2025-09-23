---
title: Set
---

# Set
The Set request inserts or updates vector entries inside a store. Each entry is defined by:

* **StoreKey**: the vector itself (list of floats).

* **StoreValue**: metadata (key-value pairs) describing the vector.

* **Input**:

  * `store`: the store name.

  * `inputs`: list of entries (StoreKey, StoreValue).

* **Behavior**: If the vector already exists, it updates the metadata. Otherwise, it inserts a new entry.

* **Response**: A confirmation response indicating success.

<details>
  <summary>Click to expand source code</summary>

```py
import asyncio
from importlib.metadata import metadata
from ahnlich_client_py.grpc import keyval, metadata
from grpclib.client import Channel
from ahnlich_client_py.grpc.services.db_service import DbServiceStub
from ahnlich_client_py.grpc.db import query as db_query
from ahnlich_client_py.grpc.db.server import Set

async def set():
  async with Channel(host="127.0.0.1", port=1369) as channel:
    client = DbServiceStub(channel)

  store_key = keyval.StoreKey(key=[5.0, 3.0, 4.0, 3.9, 4.9])
  store_value = keyval.StoreValue(
    value={"rank": metadata.MetadataValue(raw_string="chunin")}
  )

  response = await client.set(
    db_query.Set(
      store="test store",
      inputs=[keyval.DbStoreEntry(key=store_key, value=store_value)]
    )
  )

if __name__ == "__main__":
  asyncio.run(set())
```
</details>