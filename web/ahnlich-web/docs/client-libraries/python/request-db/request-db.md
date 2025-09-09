---
title: Request DB
sidebar_posiiton: 2
---

The Ahnlich DB Client is the foundation of the system, designed to efficiently manage, query, and retrieve vector embeddings. It provides all the tools needed to build applications that rely on vector similarity, metadata filtering, and structured querying.

In simple terms:

* **DB Service**: Where your vectors are stored, indexed, and retrieved.

* It supports CRUD operations, predicate-based queries, and advanced indexing strategies for optimal performance.

## Core DB Features

### Stores – Logical Containers for Vectors

A store is the primary container for vectors. Each store groups together embeddings and their metadata. You can think of it as a dataset or collection, tailored for a specific use case (e.g., product catalog, documents, images).

### Vector CRUD Operations

* **Create / Insert** vectors into a store.

* **Get** vectors by key or predicate conditions.

* **Delete** vectors by key or predicate.

This gives you full lifecycle control over your vector data.

### Predicate Filtering

The DB supports predicate-based filtering. This allows you to retrieve or delete entries that match certain metadata conditions (e.g., job = "sorcerer").

### Indexing

For efficiency, the DB provides indexing strategies:

* **Predicate Indexes** → Speed up queries on metadata conditions.

* **Non-Linear Algorithm Indexes** → Enable optimized similarity searches (e.g., KD-Tree).

Indexes ensure that even at scale, searches and lookups remain fast.

### Tracing and Observability

All DB requests support metadata injection such as trace IDs. This makes it easy to track and monitor requests in distributed systems.

## Example DB Requests

The DB client exposes a variety of request methods. Some of the most common include:

* **Ping** - Check service availability.

* **Get by Key** - Retrieve specific embeddings.

* **Get by Predicate** - Query vectors based on metadata conditions.

* **Create / Drop Indexes** - Manage performance optimizations.

* **Delete by Key or Predicate** - Remove entries from a store.

Each operation is asynchronous, ensuring high throughput for large-scale applications.

## Summary

The Ahnlich DB Client provides:

* A structured way to store and organize embeddings into logical stores.

* CRUD operations on vectors with rich filtering capabilities.

* Predicate and algorithm indexing for performance at scale.

* Built-in tracing support for observability.

It serves as the data backbone of Ahnlich, working hand-in-hand with the AI client to deliver semantic search and retrieval.

Below is a break down common DB request examples:

* [Ping](/docs/client-libraries/python/request-db/ping)
* [Info Server](/docs/client-libraries/python/request-db/info-server)
* [List Stores](/docs/client-libraries/python/request-db/list-stores)
* [Create Store](/docs/client-libraries/python/request-db/create-store)
* [Set](/docs/client-libraries/python/request-db/set)
* [GetSimN](/docs/client-libraries/python/request-db/get-simn)
* [Get Key](/docs/client-libraries/python/request-db/get-key)
* [Get By Predicate](/docs/client-libraries/python/request-db/get-by-predicate)
* [Create Predicate Index](/docs/client-libraries/python/request-db/create-predicate-index)
* [Drop Predicate Index](/docs/client-libraries/python/request-db/drop-predicate-index)
* [Delete Key](/docs/client-libraries/python/request-db/delete-key)
* [Drop Store](/docs/client-libraries/python/request-db/drop-store)
* [Create Non Linear Algorithm Index](/docs/client-libraries/python/request-db/create-non-linear-algx)
* [Drop Non Linear Algorithm Index](/docs/client-libraries/python/request-db/drop-non-linear-algx)
* [Delete Predicate](/docs/client-libraries/python/request-db/delete-predicate)