---
title: Get by Predicate
---

# Get by Predicate

**GetPred** works similarly to `Get_key`, but instead of querying by a single key, it returns results that match the defined conditions. This allows filtering AI store entries by metadata values.

* `store` – Name of the AI store to query.

* `condition` – Predicate condition that defines which entries to return. This can include equality, range, or custom predicate logic.

The result contains a list of entries matching the predicate.

<details>
  <summary>Click to expand source code</summary>

  ```py
  import asyncio
  from grpclib.client import Channel
  from ahnlich_client_py.grpc.services.ai_service import AiServiceStub
  from ahnlich_client_py.grpc.ai import query as ai_query
  from ahnlich_client_py.grpc import predicates, metadata


  async def get_by_predicate():
    async with Channel(host="127.0.0.1", port=1370) as channel:
        client = AiServiceStub(channel)
        condition = predicates.PredicateCondition(
            value=predicates.Predicate(
                equals=predicates.Equals(
                    key="brand",
                    value=metadata.MetadataValue(raw_string="Nike")
                )
            )
        )
        response = await client.get_pred(
            ai_query.GetPred(
                store="test store 1",
                condition=condition
            )
        )
        print(response) #Get(entries=[GetEntry(key=StoreInput(raw_string='Jordan One'), value=StoreValue(value={'brand': MetadataValue(raw_string='Nike')}))])


  if __name__ == "__main__":
    asyncio.run(get_by_predicate())
  ```
</details>

* The predicate condition can be extended to other metadata fields beyond "`brand`".

* This request is specifically designed for AI store queries.
