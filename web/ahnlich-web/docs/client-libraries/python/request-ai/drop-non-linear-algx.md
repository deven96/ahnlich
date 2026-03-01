---
title: Drop Non-Linear Algorithm Index
---

# Drop Non-Linear Algorithm Index

The `Drop Non Linear Algorithm` Index operation removes an index that was previously created for non-linear algorithms such as KD-Tree or HNSW. These indices are typically used to accelerate similarity searches in high-dimensional spaces.

This operation is useful when:
* An index is no longer needed and you want to free up system resources.

* You need to replace an index with a different algorithm type.

* You are resetting the store to a clean state.

If the specified index does not exist, the request will fail if `error_if_not_exists=True` is set. Otherwise, the call will safely complete without error.

<details>
  <summary>Click to expand source code</summary>

  ```py
  import asyncio
  from grpclib.client import Channel
  from ahnlich_client_py.grpc.services.ai_service import AiServiceStub
  from ahnlich_client_py.grpc.ai import query as ai_query
  from ahnlich_client_py.grpc.algorithm.nonlinear import NonLinearAlgorithm


  async def drop_non_linear_algorithm_index():
    async with Channel(host="127.0.0.1", port=1370) as channel:
        client = AiServiceStub(channel)
        response = await client.drop_non_linear_algorithm_index(
            ai_query.DropNonLinearAlgorithmIndex(
                store="test store",
                non_linear_indices=[NonLinearAlgorithm.KDTree],
                error_if_not_exists=True
            )
        )
        print(response) # Del(deleted_count=1)


  if __name__ == "__main__":
    asyncio.run(drop_non_linear_algorithm_index())
  ```
</details>

## Behavior
* **Index exists** → The specified algorithm index is removed from the store.

* **Index does not exist** →

  * If `error_if_not_exists=True`, the operation raises an error.

  * If `error_if_not_exists=False`, the request completes successfully without changes.