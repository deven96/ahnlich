---
title: Create Non-Linear algorithm Index
---

# Create Non-Linear algorithm Index

The `Create Non Linear Algorithm Index` operation builds an index structure for non-linear search algorithms, such as KD-Tree or HNSW. These indexes enable faster query performance in high-dimensional vector spaces by avoiding brute-force scans.

This operation is typically used when:
* You want to optimize search performance for similarity lookups.

* You are initializing a new store and need efficient query structures.

Each index type is specified using a `NonLinearIndex` message with either a `KDTreeConfig` or `HNSWConfig`.


<details>
  <summary>Click to expand source code</summary>

  ```py
  import asyncio
  from grpclib.client import Channel
  from ahnlich_client_py.grpc.services.ai_service import AiServiceStub
  from ahnlich_client_py.grpc.ai import query as ai_query
  from ahnlich_client_py.grpc.algorithm.nonlinear import NonLinearIndex, KDTreeConfig, HNSWConfig


  async def create_non_linear_algorithm_index():
    async with Channel(host="127.0.0.1", port=1370) as channel:
        client = AiServiceStub(channel)

        # Create a KDTree index
        response = await client.create_non_linear_algorithm_index(
            ai_query.CreateNonLinearAlgorithmIndex(
                store="test store",
                non_linear_indices=[NonLinearIndex(kdtree=KDTreeConfig())],
            )
        )
        print(response) # CreateIndex(created_indexes=1)

        # Or create an HNSW index (with optional config)
        response = await client.create_non_linear_algorithm_index(
            ai_query.CreateNonLinearAlgorithmIndex(
                store="test store",
                non_linear_indices=[NonLinearIndex(hnsw=HNSWConfig())],
            )
        )


  if __name__ == "__main__":
    asyncio.run(create_non_linear_algorithm_index())

  ```
</details>

## Behavior
* **Index does not exist** - The index for the given algorithm(s) is created successfully.

* **Index already exists** - The request completes without creating a duplicate index (idempotent).


## Notes
* Non-linear indexes are designed to improve query performance but may require additional memory.

* You can create indices for multiple algorithms by listing them under `non_linear_indices=[...]`.

* This operation only creates the index; it does not insert or modify store data.
