---
title: Drop Store
---

# Drop Store

The Drop Store request permanently deletes a store and all its contents.  
Use this carefully, as the operation is destructive and cannot be undone.

* **Behavior:** Removes the store and its data from the DB engine.

* **Response:** Confirmation response indicating deletion.

<details>
  <summary> Click to Expand Source Code</summary>

  ```py
  import asyncio
  from grpclib.client import Channel
  from grpclib.exceptions import GRPCError
  from ahnlich_client_py.grpc.services.db_service import DbServiceStub
  from ahnlich_client_py.grpc.db import query as db_query
  from ahnlich_client_py.grpc.db.server import Del


  async def drop_store():
    async with Channel(host="127.0.0.1", port=1369) as channel:
        client = DbServiceStub(channel)
        
        response = await client.drop_store(
            db_query.DropStore(
                store="test store",
                error_if_not_exists=True
            )
        )
        # response contains deleted_count
  if __name__ == "__main__":
    asyncio.run(drop_store())
  ```
</details>