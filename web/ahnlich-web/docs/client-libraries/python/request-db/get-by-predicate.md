---
title: Get by Predicate
---

# Get by Predicate

The **GetPred** request retrieves entries based on **metadata conditions (predicates)**.
Instead of matching vectors, it filters results using key/value metadata.

* **Input**:
  * `store`: store name.
  * `condition`: a predicate condition, e.g. `equals`, `greater_than`, etc.

* **Behavior**: Evaluates the predicate against metadata fields and returns all entries that satisfy the condition.

* **Response**: List of matching entries.


<details>
  <summary>Click to expand source code</summary>

  ```py
  import asyncio
  from grpclib.client import Channel
  from ahnlich_client_py.grpc.services.db_service import DbServiceStub
  from ahnlich_client_py.grpc.db import query as db_query
  from ahnlich_client_py.grpc import predicates, metadata


  async def get_predicate():
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
      
      response = await client.get_pred(
        db_query.GetPred(
          store="test store",
          condition=condition
        )
      )
      # response.entries contains matching items
    print(response)

    if __name__ == "__main__":
      asyncio.run(get_predicate())
  ```
</details>