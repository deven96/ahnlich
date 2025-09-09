---
title: List Stores
---

# List Stores

How to request a **list of available vector stores** from the Ahnlich AI Service using the Python client.

In Ahnlich, vector stores are the fundamental units that organize data for semantic search, embeddings, and AI-driven retrieval. The **List Stores** request allows developers to discover which stores are currently registered and available to query.

## Source Code Example
In the context of the rest of the application code:

<details>
  <summary>Click to expand source code</summary>

  ```py
  import asyncio
  from grpclib.client import Channel
  from ahnlich_client_py.grpc.services.ai_service import AiServiceStub
  from ahnlich_client_py.grpc.ai import query as ai_query


  async def list_stores():
    async with Channel(host="127.0.0.1", port=1370) as channel:
        client = AiServiceStub(channel)
        response = await client.list_stores(ai_query.ListStores())
        print(response) #StoreList(stores=[AiStoreInfo(name='test store', embedding_size=384)])


  if __name__ == "__main__":
    asyncio.run(list_stores())
  ```
</details>

## Define Request Parameters

The `ListStores` request does not take any required parameters.
It queries the **AI service registry** and returns metadata about all accessible stores.

## Define Response Handling

The response provides a structured list of stores, where each entry typically contains:

* **Store name** (unique identifier)

* **Configuration details** (embedding dimensions, indexing strategy, etc.)

* **Associated algorithms** (if applicable)

This allows developers to dynamically discover stores at runtime without hardcoding store names.

## Customize Usage

`ListStores` is useful for:

* **Dynamic discovery**: Applications can adapt to whatever stores exist at runtime.

* **Debugging**: Confirming that a store was successfully created and registered.

* **Observability**: Displaying available stores in admin dashboards.

The **List Stores** request is often used as a precursor to **querying embeddings** or similarity search, since it ensures the target store exists before making downstream calls.


