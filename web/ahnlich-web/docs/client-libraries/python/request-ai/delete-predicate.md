---
title: Delete Predicate
---

# Delete Predicate

This request removes all entries in an AI store that match a specified predicate condition.

* **Input:**

  * `store`: the name of the AI store.

  * `condition`: a logical predicate that filters which entries should be deleted.

* **Behavior:** This is a passthrough operation to the underlying DB service. Instead of deleting by a specific key, the server scans the store and deletes all entries that satisfy the predicate condition. In this example, it deletes all entries where the metadata field "`category`" equals "`archived`".

* **Response:**
  * `deleted_count` → the number of items successfully deleted.

<details>
  <summary>Click to expand source code</summary>

  ```py
  import asyncio
  from grpclib.client import Channel
  from ahnlich_client_py.grpc.services.ai_service import AiServiceStub
  from ahnlich_client_py.grpc.ai import query as ai_query
  from ahnlich_client_py.grpc import predicates, metadata
  from ahnlich_client_py.grpc.ai.server import Del




  async def delete_predicate():
    async with Channel(host="127.0.0.1", port=1370) as channel:
        client = AiServiceStub(channel)
        
        condition = predicates.PredicateCondition(
            value=predicates.Predicate(
                equals=predicates.Equals(
                    key="category",
                    value=metadata.MetadataValue(raw_string="archived")
                )
            )
        )
        
        response = await client.del_pred(
            ai_query.DelPred(
                store="my_ai_store",
                condition=condition
            )
        )
        # response.deleted_count shows how many items were deleted
  if __name__ == "__main__":
    asyncio.run(delete_predicate())
  ```
</details>
