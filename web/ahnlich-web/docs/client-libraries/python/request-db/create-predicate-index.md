---
title: Create Predicate Index
---

# Create Predicate Index

The CreatePredIndex request creates an index on one or more metadata fields.  
Indexes make predicate queries (e.g. GetPred) faster and more efficient.

* **Input**:

  * `store`: store name.

  * `predicates`: list of metadata fields to index.

* **Behavior**: Adds a new index on the specified fields.

* **Response**: Returns how many indexes were successfully created.  

<details>
  <summary>Click to expand source code</summary>

  ```py
  import asyncio
  from grpclib.client import Channel
  from ahnlich_client_py.grpc.services.db_service import DbServiceStub
  from ahnlich_client_py.grpc.db import query as db_query


  async def create_predicate_index():
    async with Channel(host="127.0.0.1", port=1369) as channel:
      client = DbServiceStub(channel)
      
      response = await client.create_pred_index(
        db_query.CreatePredIndex(
          store="test store",
          predicates=["job", "rank"]
        )
      )
      # response.created_indexes shows how many indexes were created
    print(response)


  if __name__ =="__main__":
    asyncio.run(create_predicate_index())
  ```
</details>