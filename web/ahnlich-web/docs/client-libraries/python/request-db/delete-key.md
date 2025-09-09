---
title: Delete Key
---

# Delete Key

This request removes one or more **keys** (and their associated values) from a store.

* **Input:**

  * `store`: the name of the store.

  * `keys`: a list of vectors (wrapped in StoreKey) that identify the entries to be deleted.

* **Behavior:** The server looks up the given keys in the store. If found, the entries are removed. If a key does not exist, it is ignored.

* **Response:**

  * `deleted_count` → the number of keys successfully deleted.  

<details>
  <summary>Click to expand source code</summary>

  ```py
  import asyncio
  from grpclib.client import Channel
  from ahnlich_client_py.grpc.services.db_service import DbServiceStub
  from ahnlich_client_py.grpc.db import query as db_query
  from ahnlich_client_py.grpc import keyval


  async def delete_key():
    async with Channel(host="127.0.0.1", port=1369) as channel:
        client = DbServiceStub(channel)
        
        store_key = keyval.StoreKey(key=[5.0, 3.0, 4.0, 3.9, 4.9])
        
        response = await client.del_key(
            db_query.DelKey(
                store="test store 002",
                keys=[store_key]
            )
        )
        # response.deleted_count shows how many items were deleted
    print(response)
          


  if __name__ == "__main__":
    asyncio.run(delete_key())
  ```
</details>