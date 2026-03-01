---
title: Drop Non-Linear Algorithm Index
---

# Drop Non-Linear Algorithm Index
This request removes one or more **non-linear algorithm indexes** from a store.  
Non-linear indexes (like **KD-Tree** and **HNSW**) are used to accelerate similarity searches.

**Input:**
  * `store`: the name of the store.

  * `non_linear_indices`: list of algorithms to drop (e.g., `KDTree`).

  * `error_if_not_exists`:

    * `True`: raises an error if the index does not exist.

    * `False`: silently ignores missing indexes.

* **Behavior:** The server attempts to remove the specified algorithm indexes from the store.

* **Response:**

  * `deleted_count` - the number of indexes successfully removed.  

<details>
  <summary>Click to expand source code</summary>

  ```py
  import asyncio
  from grpclib.client import Channel
  from ahnlich_client_py.grpc.services.db_service import DbServiceStub
  from ahnlich_client_py.grpc.db import query as db_query
  from ahnlich_client_py.grpc.algorithm.nonlinear import NonLinearAlgorithm


  async def drop_non_linear_algo():
    async with Channel(host="127.0.0.1", port=1369) as channel:
        client = DbServiceStub(channel)
        
        response = await client.drop_non_linear_algorithm_index(
            db_query.DropNonLinearAlgorithmIndex(
                store="test store 003",
                non_linear_indices=[NonLinearAlgorithm.KDTree],
                error_if_not_exists=True
            )
        )
    # response.deleted_count shows how many indexes were removed
    print(response)


  if __name__ == "__main__":
    asyncio.run(drop_non_linear_algo())
  ```
</details>

**When to use**:
* If you want to **rebuild indexes** with a different algorithm.
* If an index is no longer needed and you want to **free resources**.