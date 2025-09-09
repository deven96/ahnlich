---
title: Delete Predicate
---

# Delete Predicate

This request removes all entries in a store that match a specified predicate condition.

* **Input:**

  * `store`: the name of the store.

  * `condition`: a logical predicate that filters which entries should be deleted.

* **Behavior:** Instead of deleting by a specific key, the server scans the store and deletes all entries that satisfy the predicate condition. In this example, it deletes all entries where the metadata field "`job`" equals "`sorcerer`".

* **Response:**
  * `deleted_count` → the number of items successfully deleted.

<details>
  <summary>Click to expand source code</summary>

  ```py
  import asyncio
  from grpclib.client import Channel
  from ahnlich_client_py.grpc.services.db_service import DbServiceStub
  from ahnlich_client_py.grpc.db import query as db_query
  from ahnlich_client_py.grpc import predicates, metadata
  from ahnlich_client_py.grpc.db.server import Del




  async def delete_predicate():
    async with Channel(host="127.0.0.1", port=1369) as channel:
        client = DbServiceStub(channel)
        
        condition = predicates.PredicateCondition(
            value=predicates.Predicate(
                equals=predicates.Equals(
                    key="job",
                    value=metadata.MetadataValue(raw_string="sorcerer")
                )
            )
        )
        
        response = await client.del_pred(
            db_query.DelPred(
                store="test store 003",
                condition=condition
            )
        )
        # response.deleted_count shows how many items were deleted
  if __name__ == "__main__":
    asyncio.run(delete_predicate())
  ```
</details>