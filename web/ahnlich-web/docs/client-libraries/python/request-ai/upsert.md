---
title: Upsert
---

# Upsert

## Schema

This request accepts an optional `schema` field. When it is omitted, the server uses the `public` schema. Set `schema` to target a store in another schema.

The `Upsert` request updates a single entry matching a predicate condition in an AI store.

The AI service automatically merges metadata, preserving AI-generated fields.

<details>
  <summary>Click to expand source code</summary>

  ```py
  import asyncio
  from grpclib.client import Channel
  from ahnlich_client_py.grpc.services.ai_service import AiServiceStub
  from ahnlich_client_py.grpc.ai import query as ai_query
  from ahnlich_client_py.grpc import keyval, metadata, predicates
  from ahnlich_client_py.grpc.ai import preprocess


  async def upsert():
    async with Channel(host="127.0.0.1", port=1370) as channel:
        client = AiServiceStub(channel)
        
        condition = predicates.PredicateCondition(
          value=predicates.Predicate(
            equals=predicates.Equals(
              key="filename",
              value=metadata.MetadataValue(raw_string="photo.jpg")
            )
          )
        )
        
        new_value = keyval.StoreValue(
          value={"tags": metadata.MetadataValue(raw_string="cat,outdoors")}
        )
        
        response = await client.upsert(
            ai_query.Upsert(
                store="images",
                schema="media",
                condition=condition,
                new_input=None,  # Optional: new image/text to re-embed
                new_value=new_value,
                preprocess_action=preprocess.PreprocessAction.NoPreprocessing,
                execution_provider=None,
                model_params={}
            )
        )
        print(response) #Set(upsert=StoreUpsert(updated=1, inserted=0))


  if __name__ == "__main__":
    asyncio.run(upsert())

  ```
</details>

## Key Notes

* **`condition`** - predicate that must match exactly one entry.

* **`new_input`** (optional) - new raw input to re-embed (e.g., updated text, image, or audio).

* **`new_value`** (optional) - metadata to update. Always merged with existing metadata.

* **`preprocess_action`** - how inputs are preprocessed before embedding.

* **`execution_provider`** - Optional hardware acceleration (e.g., `CUDA`).

* **`model_params`** - Optional runtime parameters for the AI model.

* **Behavior** - AI proxy always merges metadata, preserving AI-generated fields. Errors if 0 or multiple entries match.

* **Response** → returns upsert counts (inserted: 0, updated: 1).
