---
title: Drop Predicate Index
---

# Drop Predicate Index

Predicate index allow for efficient querying based on metadata fields. You can drop either a **single predicate index** or **multiple predicate indices** depending on your needs.

* **Single Predicate Index**: Specify a single predicate in the `predicates` list, e.g., `["job"]`. This will remove the index associated with that one metadata field.


* **Multiple Predicate Indices**: Include multiple predicates in the list, e.g., `["job", "rank"]`. All listed indices will be dropped in a single request.

**Note**: The `error_if_not_exists` flag ensures that an error is thrown if the index does not exist. The `response.deleted_count` property shows how many indices were actually removed.


<details>
  <summary>Click to expand source code</summary>

  ```py
  import asyncio
  from grpclib.client import Channel
  from ahnlich_client_py.grpc.services.ai_service import AiServiceStub
  from ahnlich_client_py.grpc.ai import query as ai_query


  async def drop_predicate_index():
    async with Channel(host="127.0.0.1", port=1370) as channel:
        client = AiServiceStub(channel)
        response = await client.drop_pred_index(
            ai_query.DropPredIndex(
                store="test store",
                predicates=["job"],
                error_if_not_exists=True
            )
        )
        print(response) # Del(deleted_count=1)


  if __name__ == "__main__":
    asyncio.run(drop_predicate_index())
  ```
</details>

This approach allows you to maintain flexibility, dropping either one or many predicate indices in a single operation, while keeping your store optimized for AI queries.