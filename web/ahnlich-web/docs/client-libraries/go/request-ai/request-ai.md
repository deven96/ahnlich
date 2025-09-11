---
title: Request AI
sidebar_posiiton: 3
---

# Request - AI

Ahnlich Request-ai: AI proxy to communicate with ahnlich-db, receiving raw input, transforming it into embeddings, and storing those embeddings within the DB. It extends DB capabilities by allowing developers to issue queries to the same store using raw inputs such as images or text. The AI proxy features multiple off-the-shelf models which can be selected for both store indexing and query time.

Ahnlich Request-AI acts as a bridge between raw developer inputs (text, images, etc.) and the vector store. Instead of requiring clients to precompute embeddings, the AI proxy accepts raw inputs, runs a model to create embeddings, and persists those embeddings into the target Ahnlich DB store. Once stored, the same store can be queried using raw input — the proxy will convert the query into an embedding and run the search on the DB.

## When to use Ahnlich Request AI
- Simplify client logic: let the proxy handle embedding generation so clients send raw content (text/images) rather than precomputed vectors.

- Faster prototyping: quickly add semantic search by selecting an off-the-shelf model without changing client code.

- Model selection flexibility: pick different models for indexing and querying to suit accuracy/latency tradeoffs.


## How it works 
1. Receive raw input — the AI proxy accepts raw payloads (for example, text or image data).

2. Transform to embeddings — the proxy runs the selected model to generate vector embeddings for the input.

3. Store in DB — embeddings plus any associated metadata are stored in the specified Ahnlich DB store.


4. Query with raw input — when you issue a query using raw input, the proxy repeats steps 1–3 for the query and forwards results.


## Model configuration
When creating or configuring a store via the AI proxy, you supply two model identifiers:

- INDEXMODEL — the model used to generate embeddings for items that will be stored (indexing).

- QUERYMODEL — the model used to transform incoming raw queries into embeddings for search.

*Example (CLI-style):*
```
CREATESTORE my_store QUERYMODEL all-minilm-l6-v2 INDEXMODEL all-minilm-l6-v2
```

Structured example (provided):
```
create_store(
    store="my_store",
    index_model="all-minilm-l6-v2",
    query_model="all-minilm-l6-v2",
)
```

## Behavior and expectations

- The AI proxy generates embeddings on behalf of the client and persists them into the DB, so clients do not need to manage embedding computation.

- The proxy supports multiple off-the-shelf models; you select the model identifiers when creating or configuring a store.

- After indexing, the same store can be queried using raw input; the proxy will convert the query into an embedding using the configured `query_model` and forward the similarity request to the DB.

- Choosing different `index_model` and `query_model` is supported (the example uses the same model for both), enabling flexibility in balancing index-time embedding quality vs. query-time performance.


## Source Code Example
Create a store named my_store and select the same model for both indexing and querying:
```
CREATESTORE my_store QUERYMODEL all-minilm-l6-v2 INDEXMODEL all-minilm-l6-v2
```

Equivalent in structured form:
```
create_store(
    store="my_store",
    index_model="all-minilm-l6-v2",
    query_model="all-minilm-l6-v2",
)
```

Below is a breakdown of common AI request examples:



* [Ping](/docs/client-libraries/go/request-db/ping)
* [Info Server](/docs/client-libraries/go/request-db/info-server)
* [List Stores](/docs/client-libraries/go/request-db/list-stores)
* [Create Store](/docs/client-libraries/go/request-db/create-store)
* [Set](/docs/client-libraries/go/request-db/set)
* [GetSimN](/docs/client-libraries/go/request-db/get-simn)
* [Get By Predicate](/docs/client-libraries/go/request-db/get-by-predicate)
* [Create Predicate Index](/docs/client-libraries/go/request-db/create-predicate-index)
* [Drop Predicate Index](/docs/client-libraries/go/request-db/drop-predicate-index)
* [Delete Key](/docs/client-libraries/go/request-db/delete-key)
* [Drop Store](/docs/client-libraries/go/request-db/drop-store)
* [Create Non Linear Algorithm Index](/docs/client-libraries/go/request-db/create-non-linear-algx)
* [Drop Non Linear Algorithm Index](/docs/client-libraries/go/request-db/drop-non-linear-algx)
<!-- * [Get Key](/docs/client-libraries/go/request-db/get-key) -->
<!-- * [Delete Predicate](/docs/client-libraries/go/request-db/delete-predicate) -->

