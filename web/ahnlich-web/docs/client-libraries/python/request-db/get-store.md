---
title: Get Store
---

# Get Store

Returns detailed information about a specific store by name.

* **Input**: Store name.

* **Behavior**: Retrieves metadata and configuration for the specified store.

* **Response**: Store information including name, size, dimension, and indices.

<details>
  <summary>Click to expand source code</summary>

```py
import asyncio
from grpclib.client import Channel
from ahnlich_client_py.grpc.services.db_service import DbServiceStub
from ahnlich_client_py.grpc.db import query as db_query


async def get_store_info():
    async with Channel(host="127.0.0.1", port=1369) as channel:
        client = DbServiceStub(channel)

        response = await client.get_store(
            db_query.GetStore(store="my_store")
        )

        print(f"Store name: {response.name}")
        print(f"Number of entries: {response.len}")
        print(f"Size in bytes: {response.size_in_bytes}")
        print(f"Dimension: {response.dimension}")
        print(f"Predicate indices: {response.predicate_indices}")
        print(f"Non-linear indices: {response.non_linear_indices}")


if __name__ == "__main__":
    asyncio.run(get_store_info())
```
</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `store` | `str` | Yes | The name of the store to retrieve |

## Response: StoreInfo

| Field | Type | Description |
|-------|------|-------------|
| `name` | `str` | Store name |
| `len` | `int` | Number of entries in the store |
| `size_in_bytes` | `int` | Total size of the store in bytes |
| `dimension` | `int` | Vector dimension |
| `predicate_indices` | `List[str]` | List of indexed predicate keys |
| `non_linear_indices` | `List[NonLinearIndex]` | List of non-linear algorithm indices (KDTree, HNSW) |

## Notes

- Returns an error if the store does not exist
- Use `ListStores` to get information about all stores
- The `size_in_bytes` field is useful for monitoring memory usage
