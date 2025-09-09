---
title: Drop Store
---

# Drop Store

The **Drop Store** request removes an entire AI store and all its contents.
Use this when you no longer need the vector store or want to clean up your environment.

**Inputs**:
* `store` – Name of the AI store to drop.

* `error_if_not_exists` – If `True`, the request will fail if the store does not exist.

**Response**: Returns a response containing the number of stores deleted (`deleted_count`).

<details>
  <summary>Click to expand source code</summary>

  ```py
  import asyncio
  from grpclib.client import Channel
  from ahnlich_client_py.grpc.services.ai_service import AiServiceStub
  from ahnlich_client_py.grpc.ai import query as ai_query


  async def drop_store():
    async with Channel(host="127.0.0.1", port=1370) as channel:
        client = AiServiceStub(channel)
        response = await client.drop_store(
            ai_query.DropStore(
                store="test store",
                error_if_not_exists=True
            )
        )
        print(response) # Del(deleted_count=1)


  if __name__ == "__main__":
    asyncio.run(drop_store())
  ```
</details>