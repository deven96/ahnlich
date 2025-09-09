---
title: GetKey
---

# GetKey

The GetKey request retrieves entries from a store based on an exact vector key match.

* **Input**:
  * `store`: the store name.
  * `key`: the exact vector you want to retrieve.

* **Behavior**: Finds the stored entry that matches the vector key exactly.

* **Response**: Returns the entry (vector + metadata) if found.

<details>
  <summary>Click to expand source code</summary>

  ```py
    import asyncio
    from grpclib.client import Channel
    from ahnlich_client_py.grpc import keyval
    from ahnlich_client_py.grpc.services.db_service import DbServiceStub
    from ahnlich_client_py.grpc.db import query as db_query
    from ahnlich_client_py.grpc.db.server import Get

    async def get_key():
      async with Channel(host="127.0.0.1", port=1369) as channel:
        client = DbServiceStub(channel)
        
        lookup_key = keyval.StoreKey(key=[5.0, 3.0, 4.0, 3.9, 4.9])  # Your lookup vector
        
        response = await client.get_key(
          db_query.GetKey(
              store="customer_profiles",
              keys=[lookup_key]
          )
        )
        # response.entries contains matching (key, value) pairs
    if __name__ == "__main__":
      asyncio.run(get_key())

  ```
</details>
