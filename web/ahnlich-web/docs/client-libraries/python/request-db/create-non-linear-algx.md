---
title: Create Non-Linear Algorithm Index
---

# Create Non-Linear Algorithm Index

Creating non-linear algorithm indexes allows you to optimize query execution based on spatial or high-dimensional data structures.

The NonLinearAlgorithm enum currently supports algorithms like KDTree (K-dimensional tree), which is useful for nearest-neighbor searches and multidimensional range queries.

Non Linear Algorithm Indexes improve query performance by pre-structuring the data, but depending on the algorithm, there may be tradeoffs between query time and memory consumption.

In the Ahnlich client, you can create a non-linear algorithm index by calling the create_non_linear_algorithm_index RPC via the DbServiceStub.

## Define a Client and Call the API
The following example shows how to initialize a client, request index creation, and inspect the server’s response.


<details>
  <summary>Click to expand code</summary>

  ```py
  import asyncio
  from grpclib.client import Channel
  from ahnlich_client_py.grpc.services.db_service import DbServiceStub
  from ahnlich_client_py.grpc.db import query as db_query
  from ahnlich_client_py.grpc.algorithm.nonlinear import NonLinearAlgorithm


  async def create_non_linear_algo_index():
    async with Channel(host="127.0.0.1", port=1369) as channel:
        client = DbServiceStub(channel)
        
        response = await client.create_non_linear_algorithm_index(
            db_query.CreateNonLinearAlgorithmIndex(
                store="test store 003",
                non_linear_indices=[NonLinearAlgorithm.KDTree]
            )
        )
        # response.created_indexes shows how many indexes were created
    print(response)


  if __name__ == "__main__":
    asyncio.run(create_non_linear_algo_index())
  ```
</details>