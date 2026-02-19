---
title: Create Store
---

# Create Store

The `CreateStore` request is used to **initialize a new AI-powered store**.
Unlike the DB client (which deals with raw vector dimensions), the AI client lets you specify **pretrained AI models** to handle embedding generation and indexing.

This means you don’t have to manage vectors manually — the AI service will automatically embed inputs using the selected models.


<details>
  <summary>Click to expand source code</summary>

  ```py
  import asyncio
  from grpclib.client import Channel
  from ahnlich_client_py.grpc.services.ai_service import AiServiceStub
  from ahnlich_client_py.grpc.ai import query as ai_query
  from ahnlich_client_py.grpc.ai.models import AiModel


  async def create_store():
    async with Channel(host="127.0.0.1", port=1370) as channel:
        client = AiServiceStub(channel)
        response = await client.create_store(
            ai_query.CreateStore(
                store="test store",
                query_model=AiModel.ALL_MINI_LM_L6_V2,
                index_model=AiModel.ALL_MINI_LM_L6_V2,
                predicates=["job"],
                non_linear_indices=[],  # Optional: non-linear algorithms for faster search
                error_if_exists=True,
                # Store original controls if we choose to store the raw inputs
                # within the DB in order to be able to retrieve the originals again
                # during query, else only store values are returned
                store_original=True
            )
        )
        print(response) # Unit()


  if __name__ == "__main__":
    asyncio.run(create_store())
  ```
</details>

## Key Notes

* `query_model` - model used for encoding query inputs during searches.

* `index_model` - model used for encoding stored data vectors.

* `predicates` - metadata fields that can be filtered against (e.g., "`job`").

* `non_linear_indices` - list of non-linear algorithms for approximate search (can be empty list).

* `store_original` - if `True`, the original raw input is stored alongside embeddings for later retrieval.

* **Response** - returns `Unit()` on success.

This request is critical in AI workflows because it allows you to:

* Configure **semantic stores** with specialized embedding models.

* Decide whether to preserve raw input text/images for retrieval.

* Build **intelligent**, **AI-driven search** and **recommendation systems** without managing embeddings manually.

