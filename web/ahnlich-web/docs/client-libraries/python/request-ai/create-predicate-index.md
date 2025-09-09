---
title: Create Predicate Index
---

# Create Predicate Index

Predicate indices allow the AI store to efficiently filter results based on metadata fields. Use this operation to define which metadata keys should be indexed for faster query operations like `GetPred`.

* `store` – Name of the AI store to create predicate indices on.

* `predicates` – List of metadata fields to index.

The response confirms the creation of indices.


<details>
  <summary>Click to expand source code</summary>

  ```py
  import asyncio
  from grpclib.client import Channel
  from ahnlich_client_py.grpc.services.ai_service import AiServiceStub
  from ahnlich_client_py.grpc.ai import query as ai_query


  async def create_predicate_index():
    async with Channel(host="127.0.0.1", port=1370) as channel:
        client = AiServiceStub(channel)
        response = await client.create_pred_index(
            ai_query.CreatePredIndex(
                store="test store",
                predicates=["job", "rank"]
            )
        )
        print(response) # CreateIndex(created_indexes=1)


  if __name__ == "__main__":
    asyncio.run(create_predicate_index())
  ```
</details>

* Use this function after creating a store or inserting entries to optimize predicate-based queries.

* Only indexed predicates can be efficiently queried in `GetPred`.