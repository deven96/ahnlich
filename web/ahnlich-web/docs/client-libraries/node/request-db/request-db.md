---
title: Request DB
sidebar_position: 1
---

# Request DB

This section covers all available DB (Database) operations for the Node.js SDK when interacting with Ahnlich DB.

Ahnlich DB is the core vector storage engine that provides:

* Vector storage with configurable dimensions
* Similarity search using various algorithms
* Metadata filtering with predicates
* Non-linear indexing (KDTree, HNSW) for faster searches

## Available Operations

### Server Operations
- [Ping](/docs/client-libraries/node/request-db/ping) - Health check
- [Info Server](/docs/client-libraries/node/request-db/info-server) - Get server information
- [List Connected Clients](/docs/client-libraries/node/request-db/list-connected-clients) - List all connected clients

### Store Operations
- [List Stores](/docs/client-libraries/node/request-db/list-stores) - List all stores
- [Get Store](/docs/client-libraries/node/request-db/get-store) - Get details of a specific store
- [Create Store](/docs/client-libraries/node/request-db/create-store) - Create a new store
- [Drop Store](/docs/client-libraries/node/request-db/drop-store) - Delete a store

### Data Operations
- [Set](/docs/client-libraries/node/request-db/set) - Insert or update entries
- [Get Key](/docs/client-libraries/node/request-db/get-key) - Retrieve entries by key
- [GetSimN](/docs/client-libraries/node/request-db/get-simn) - Find N most similar entries
- [Get By Predicate](/docs/client-libraries/node/request-db/get-by-predicate) - Filter entries by metadata
- [Delete Key](/docs/client-libraries/node/request-db/delete-key) - Delete entries by key
- [Delete Predicate](/docs/client-libraries/node/request-db/delete-predicate) - Delete entries matching a predicate

### Index Operations
- [Create Predicate Index](/docs/client-libraries/node/request-db/create-predicate-index) - Create metadata index
- [Drop Predicate Index](/docs/client-libraries/node/request-db/drop-predicate-index) - Remove metadata index
- [Create Non Linear Algorithm Index](/docs/client-libraries/node/request-db/create-non-linear-algx) - Create KDTree/HNSW index
- [Drop Non Linear Algorithm Index](/docs/client-libraries/node/request-db/drop-non-linear-algx) - Remove KDTree/HNSW index
