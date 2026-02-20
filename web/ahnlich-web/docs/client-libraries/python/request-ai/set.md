---
title: Set
---

# Set

The `Set` request is used to **insert or update entries** inside an AI-powered store.

Unlike the DB client (which expects raw vectors), the AI client allows you to store **raw strings or other inputs** directly.
The AI service will automatically **embed these inputs** using the store’s configured models.

<details>
  <summary>Click to expand source code</summary>

  ```py
  import asyncio
  from grpclib.client import Channel
  from ahnlich_client_py.grpc.services.ai_service import AiServiceStub
  from ahnlich_client_py.grpc.ai import query as ai_query
  from ahnlich_client_py.grpc import keyval, metadata
  from ahnlich_client_py.grpc.ai import preprocess


  async def sets():
    async with Channel(host="127.0.0.1", port=1370) as channel:
        client = AiServiceStub(channel)
        response = await client.set(
            ai_query.Set(
                store="test store",
                inputs=[
                    keyval.AiStoreEntry(
                        key=keyval.StoreInput(raw_string="Jordan One"),
                        value=keyval.StoreValue(
                            value={"brand": metadata.MetadataValue(raw_string="Nike")}
                        ),
                    ),
                    keyval.AiStoreEntry(
                        key=keyval.StoreInput(raw_string="Yeezey"),
                        value=keyval.StoreValue(
                            value={"brand": metadata.MetadataValue(raw_string="Adidas")}
                        ),
                    )
                ],
                preprocess_action=preprocess.PreprocessAction.NoPreprocessing,
                execution_provider=None  # Optional: e.g., ExecutionProvider.CUDA for GPU acceleration
            )
        )
        print(response) #Set(upsert=StoreUpsert(inserted=2))


  if __name__ == "__main__":
    asyncio.run(sets())

  ```
</details>

## Key Notes

* **`inputs`** - list of entries to be stored.

  * Each entry has:

    * **`key`** - the raw input (e.g., "Jordan One") that gets embedded by the AI model.

    * **`value`** - metadata associated with the key (e.g., `"brand": Nike`).

* **`preprocess_action`** - defines how inputs are preprocessed before embedding.

  * `NoPreprocessing` - raw text is passed as-is to the embedding model.

  * Other preprocessing options (like normalization or tokenization) can be applied depending on the use case.

* **`execution_provider`** - Optional hardware acceleration for model inference (e.g., `CUDA`, `TensorRT`, `CoreML`). Set to `None` for default CPU execution.

* **Response** → returns counts of inserted vs. updated items (`upsert counts`).  