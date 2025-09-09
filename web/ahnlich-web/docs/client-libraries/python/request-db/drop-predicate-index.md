---
title: Drop Predicate Index
---

# Drop Predicate Index

The DropPredIndex request removes an index from one or more metadata fields.  
This should be used when an index is no longer needed or to reduce storage overhead.

* **Input**:

  * `store`: store name.

  * `predicates`: list of metadata fields whose indexes should be dropped.

  * `error_if_not_exists`: if `True`, raises error if the index does not exist.

* **Behavior**: Deletes the index, but the underlying data remains.

* **Response**: Returns how many indexes were deleted.

<details>
  <summary>Click to expand source code</summary>

  ```py
  import asyncio
  from grpclib.client import Channel
  from ahnlich_client_py.grpc.services.db_service import DbServiceStub
  from ahnlich_client_py.grpc.db import query as db_query
  from ahnlich_client_py.grpc.db.server import Del


  async def drop_predicate_index():
    async with Channel(host="127.0.0.1", port=1369) as channel:
        client = DbServiceStub(channel)
        
        response = await client.drop_pred_index(
            db_query.DropPredIndex(
                store="test store",
                predicates=["job"],
                error_if_not_exists=True
            )
        )
        # response.deleted_count shows how many indexes were removed
        print(response)


  if __name__ == "__main__":
    asyncio.run(drop_predicate_index())
  ```
</details>