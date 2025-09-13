---
title: Request DB
sidebar_posiiton: 2
---

# Requests – DB

The **Ahnlich DB** is an in-memory vector key–value store designed for storing embeddings or vectors alongside their metadata (key–value maps).

It provides AI/ML engineers with the ability to:

- Store and retrieve embeddings.

- Search for similar vectors using linear similarity algorithms (Cosine, Euclidean).

- Perform searches with non-linear similarity algorithms (such as KD-Tree).

- Filter results based on metadata values.

This makes it possible to build intelligent search and recommendation systems that combine vector similarity with metadata-based filtering.

_Example_

A query to retrieve the 2 most similar vectors to `[0.2, 0.1]` from the store `my_store`, using cosine similarity, while excluding any items where the metadata field `page` is equal to `"hidden"`:

```go
GETSIMN 2 WITH [0.2, 0.1] USING cosinesimilarity IN my_store WHERE (page != hidden)
```

Below is a breakdown of common DB request examples:

- [Ping](/docs/client-libraries/go/request-db/ping)
- [Info Server](/docs/client-libraries/go/request-db/info-server)
- [List Stores](/docs/client-libraries/go/request-db/list-stores)
- [Create Store](/docs/client-libraries/go/request-db/create-store)
- [Set](/docs/client-libraries/go/request-db/set)
- [GetSimN](/docs/client-libraries/go/request-db/get-simn)
- [Get Key](/docs/client-libraries/go/request-db/get-key)
- [Get By Predicate](/docs/client-libraries/go/request-db/get-by-predicate)
- [Create Predicate Index](/docs/client-libraries/go/request-db/create-predicate-index)
- [Drop Predicate Index](/docs/client-libraries/go/request-db/drop-predicate-index)
- [Delete Key](/docs/client-libraries/go/request-db/delete-key)
- [Drop Store](/docs/client-libraries/go/request-db/drop-store)
- [List Connected Clients](/docs/client-libraries/go/request-db/list-connected-clients)
- [Create Non Linear Algorithm Index](/docs/client-libraries/go/request-db/create-non-linear-algx)
- [Drop Non Linear Algorithm Index](/docs/client-libraries/go/request-db/drop-non-linear-algx)
- [Delete Predicate](/docs/client-libraries/go/request-db/delete-predicate)
