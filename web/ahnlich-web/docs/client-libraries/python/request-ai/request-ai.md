---
title: Request AI
sidebar_posiiton: 3
---

# Request AI

The **Ahnlich AI Client** provides intelligent services that extend the capabilities of the DB client.
While the DB client is optimized for storing vectors, managing stores, and retrieving them efficiently, the AI client is responsible for **embedding generation**, **preprocessing**, and **querying using raw inputs**.

## Key Features
* **Raw input support**: Accepts text or images as input.

* **Embedding generation**: Automatically transforms raw input into embeddings using AI models.

* **AI proxy**: Communicates with Ahnlich-DB to persist embeddings and metadata.

* **Query with raw input**: No need to manually generate vectors just pass text or images.

* **Flexible models**: Choose different models for indexing vs querying.

* **Metadata integration**: Store structured information alongside embeddings.

* **Bulk requests**: Chain multiple operations (`ping`, `list_stores`, etc.) for efficiency.

## Supported Raw Inputs
* **Text** - sentences, titles, product descriptions, etc.

* **Images** - passed as binary (`u8` list).

* **Mixed stores** depending on the selected AI model.

## Models
* **Index Model** - Used when adding new items (generates embeddings).

* **Query Model** - Used when searching (generates query embeddings).

* You may configure both models separately, but they should be compatible.

## Create a Store

```py
  create_store(
    store="my_store",
    index_model="all-minilm-l6-v2",
    query_model="all-minilm-l6-v2",
  )
```

## Insert Raw Input

<details>
  <summary>Expand code</summary>

  ```py
  response = await client.set(
    ai_query.Set(
        store="my_store",
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
        preprocess_action=preprocess.PreprocessAction.NoPreprocessing
    )
  )
  ```
</details>

## Query with Raw Input

<details>
  <summary>Expand code</summary>

  ```py
    response = await client.get_sim_n(
      ai_query.GetSimN(
          store="my_store",
          search_input=keyval.StoreInput(raw_string="Jordan"),
          closest_n=3,
          algorithm=algorithms.Algorithm.COSINE_SIMILARITY,
      )
  )

  for entry in response.entries:
      print(f"Key: {entry.key.raw_string}, Score: {entry.score}")
  ```
</details>

Below is a breakdown of common AI request examples:

* [Ping](/docs/client-libraries/python/request-ai/ping)
* [Info Server](/docs/client-libraries/python/request-ai/info-server)
* [List Stores](/docs/client-libraries/python/request-ai/list-stores)
* [Create Store](/docs/client-libraries/python/request-ai/create-store)
* [Set](/docs/client-libraries/python/request-ai/set)
* [GetSimN](/docs/client-libraries/python/request-ai/get-simn)
* [Get By Predicate](/docs/client-libraries/python/request-ai/get-by-predicate)
* [Create Predicate Index](/docs/client-libraries/python/request-ai/create-predicate-index)
* [Drop Predicate Index](/docs/client-libraries/python/request-ai/drop-predicate-index)
* [Delete Key](/docs/client-libraries/python/request-ai/delete-key)
* [Drop Store](/docs/client-libraries/python/request-ai/drop-store)
* [Create Non Linear Algorithm Index](/docs/client-libraries/python/request-ai/create-non-linear-algx)
* [Drop Non Linear Algorithm Index](/docs/client-libraries/python/request-ai/drop-non-linear-algx)