---
title: Purge Stores
---

# Purge Stores

Deletes **all vector stores** managed by the AI server, including all embeddings and associated metadata. This is a destructive operation that resets the AI service state, typically used during testing, cleanup, or when starting fresh with new datasets.

* **Input**: No arguments required.

* **Behavior**: Removes all stores and their contents from the AI server.

* **Response**: Confirmation of deletion with count of deleted stores.

<details>
  <summary>Click to expand source code</summary>

  ```py
  import asyncio
  from grpclib.client import Channel
  from ahnlich_client_py.grpc.services.ai_service import AiServiceStub
  from ahnlich_client_py.grpc.ai import query as ai_query

  async def purge_stores():
      async with Channel(host="127.0.0.1", port=1370) as channel:
          client = AiServiceStub(channel)
          
          response = await client.purge_stores(ai_query.PurgeStores())
          
          print(f"Purged stores. Deleted count: {response.deleted_count}")

  if __name__ == "__main__":
      asyncio.run(purge_stores())
  ```
</details>

:::warning
This operation is **irreversible**. All stores and their data will be permanently deleted. Use with caution in production environments.
:::
