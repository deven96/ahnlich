---
title: Get Store
---

# Get Store

Returns detailed information about a specific AI store by name.

* **Input**: Store name.

* **Behavior**: Retrieves metadata and configuration for the specified AI store, including model information.

* **Response**: AI store information including models, dimension, and optional DB store info.

<details>
  <summary>Click to expand source code</summary>

```py
import asyncio
from grpclib.client import Channel
from ahnlich_client_py.grpc.services.ai_service import AiServiceStub
from ahnlich_client_py.grpc.ai import query as ai_query


async def get_ai_store_info():
    async with Channel(host="127.0.0.1", port=1370) as channel:
        client = AiServiceStub(channel)

        response = await client.get_store(
            ai_query.GetStore(store="ai_store")
        )

        print(f"Store name: {response.name}")
        print(f"Query model: {response.query_model}")
        print(f"Index model: {response.index_model}")
        print(f"Embedding size: {response.embedding_size}")
        print(f"Dimension: {response.dimension}")
        print(f"Predicate indices: {response.predicate_indices}")

        if response.db_info:
            print(f"DB store size: {response.db_info.size_in_bytes} bytes")


if __name__ == "__main__":
    asyncio.run(get_ai_store_info())
```
</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `store` | `str` | Yes | The name of the AI store to retrieve |

## Response: AiStoreInfo

| Field | Type | Description |
|-------|------|-------------|
| `name` | `str` | Store name |
| `query_model` | `AiModel` | AI model used for query embeddings |
| `index_model` | `AiModel` | AI model used for index embeddings |
| `embedding_size` | `int` | Number of stored embeddings |
| `dimension` | `int` | Vector dimension (determined by model) |
| `predicate_indices` | `List[str]` | List of indexed predicate keys |
| `db_info` | `Optional[StoreInfo]` | Underlying DB store info (when AI is connected to DB) |

## Notes

- Returns an error if the store does not exist
- The `db_info` field is present when the AI proxy is connected to a DB instance
- Use `ListStores` to get information about all AI stores
