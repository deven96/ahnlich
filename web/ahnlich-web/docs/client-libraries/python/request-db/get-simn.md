---
title: GetSimN
---

# GetSimN

The GetSimN request performs a similarity search.  
It retrieves the N closest vectors to a given query vector.

* **Input**:

  * `store`: store name.

  * `search_input`: the query vector (`StoreKey`).

  * `closest_n`: number of results to return (> 0).

  * `algorithm`: similarity metric (e.g. CosineSimilarity, EuclideanDistance).

  * `condition`: optional predicate filter to restrict which vectors are considered. Set to `None` to search all vectors. See [Predicates documentation](/components/predicates/predicates) for filtering examples.

* **Behavior**: The server compares the query vector with stored vectors using the chosen similarity metric.

* **Response**: A list of entries with:

  * `key` (vector),

  * `value` (metadata),

  * `score` (similarity measure).

<details>
  <summary>Click to expand source code</summary>

  ```py
  import asyncio
  from grpclib.client import Channel
  from ahnlich_client_py.grpc.services.db_service import DbServiceStub
  from ahnlich_client_py.grpc.db import query as db_query
  from ahnlich_client_py.grpc import keyval, predicates
  from ahnlich_client_py.grpc.algorithm.algorithms import Algorithm


  async def get_simn():
    async with Channel(host="127.0.0.1", port=1369) as channel:
      client = DbServiceStub(channel)


      search_key = keyval.StoreKey(key=[5.0, 5.1, 3.4, 5.1, 4.9])


      response = await client.get_sim_n(
        db_query.GetSimN(
          store="test store",
          search_input=search_key,
          closest_n=3,
          algorithm=Algorithm.CosineSimilarity,
          condition=None  # Optional: filter results using predicates
        )
      )

      print(response.entries)  # [(key, value, score), ...]


  if __name__ == "__main__":
    asyncio.run(get_simn())
  ```
</details>