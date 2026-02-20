---
title: GetSimN
---

# GetSimN

How to retrieve the top N most similar entries from an AI store using the Ahnlich Python SDK.

**GetSimN** returns an array of tuples (`store_key`, `store_value`, `similarity_score`) of the maximum specified `N`. This allows you to perform similarity searches against the stored AI embeddings.

* `store` – Name of the AI store to query.

* `search_input` – Query input (string or vector).

* `closest_n` – Maximum number of similar results to return (must be > 0).

* `algorithm` – Similarity algorithm to use (e.g., Cosine Similarity).

* `condition` – Optional predicate condition to filter results. See [Predicates documentation](/components/predicates/predicates).

* `preprocess_action` – Controls input preprocessing:
  - `PreprocessAction.ModelPreprocessing` – Apply model's built-in preprocessing (recommended)
  - `PreprocessAction.NoPreprocessing` – Skip preprocessing (use if you've already preprocessed the input)

* `execution_provider` – Optional hardware acceleration (e.g., CUDA, TensorRT, CoreML). Set to `None` to use default CPU execution.

The result contains a list of entries with similarity scores.

Source code in the context of the rest of the application code.

<details>
  <summary>Click to expand</summary>

  ```py
  import asyncio
  from grpclib.client import Channel
  from ahnlich_client_py.grpc.services.ai_service import AiServiceStub
  from ahnlich_client_py.grpc.ai import query as ai_query
  from ahnlich_client_py.grpc import keyval
  from ahnlich_client_py.grpc.algorithm import algorithms
  from ahnlich_client_py.grpc.ai.preprocess import PreprocessAction


  async def get_sim_n():
    async with Channel(host="127.0.0.1", port=1370) as channel:
        client = AiServiceStub(channel)
        
        response = await client.get_sim_n(
            ai_query.GetSimN(
                store="test store 1",
                search_input=keyval.StoreInput(raw_string="Jordan"),
                condition=None,  # Optional predicate condition
                closest_n=3,
                algorithm=algorithms.Algorithm.CosineSimilarity,
                preprocess_action=PreprocessAction.ModelPreprocessing,  # Apply model's preprocessing
                execution_provider=None  # Optional execution provider
            )
        )
        
        # Response contains entries with similarity scores
        for entry in response.entries:
            print(f"Key: {entry.key.raw_string}")
            print(f"Score: {entry.similarity}")
            print(f"Value: {entry.value}")


        # Key: Jordan One
        # Score: Similarity(value=0.858908474445343)
        # Value: StoreValue(value={'brand': MetadataValue(raw_string='Nike')})
        # Key: Yeezey
        # Score: Similarity(value=0.21911849081516266)
        # Value: StoreValue(value={'brand': MetadataValue(raw_string='Adidas')})


  if __name__ == "__main__":
    asyncio.run(get_sim_n())
  ```
</details>

* `closest_n` must always be a non-zero integer.

* This request is designed specifically for AI store queries.
