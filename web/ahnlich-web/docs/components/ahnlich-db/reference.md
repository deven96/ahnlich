---
title: Reference
sidebar_position: 30
---

# Reference

This section documents all supported DB request commands in Ahnlich DB. Commands can be executed via the text-based query interface or programmatically through client APIs.

### 1. Server & System
#### PING
Check server health.
```
PING
```

Returns `PONG` if the server is running.

#### INFO SERVER
Retrieve server information (version, uptime, memory usage).
```
INFO SERVER
```

#### LIST CONNECTED CLIENTS
View currently connected clients.
```
LIST CONNECTED CLIENTS
```


### 2. Store Management

#### LIST STORES
Show all existing vector stores.
```
LIST STORES
```

#### CREATE STORE
Create a new store with a given dimension and algorithm.
```
CREATE STORE <store_name> DIMENSION <n> ALGORITHM <cosine|euclidean|kdtree>
```

#### Example:
```
CREATE STORE my_store DIMENSION 128 ALGORITHM cosine
```

#### DROP STORE
Remove a store and its contents.
```
DROP STORE <store_name>
```

#### Example:
```
DROP STORE my_store
```


### 3. Insert, Update & Delete
#### SET
Insert or update a vector with metadata.
```
SET <key> [<float>, <float>, ...] WITH { "<meta_key>": "<meta_value>", ... } IN <store_name>
```

#### Example:
```
SET doc1 [0.25, 0.88] WITH { "category": "news", "lang": "en" } IN my_store
```

#### DELETE KEY
Delete a vector by key.
```
DELETE KEY <key> IN <store_name>
```

#### Example:
```
DELETE KEY doc1 IN my_store
```

#### DELETE PREDICATE
Delete all vectors that match a predicate.
```
DELETE PREDICATE <predicate> IN <store_name>
```

#### Example:
```
DELETE PREDICATE (category = "archive") IN my_store
```


### 4. Query & Retrieval
#### GET SIM N
Find the N most similar vectors to an input vector.
```
GETSIMN <n> WITH [<float>, <float>, ...] USING <cosinesimilarity|euclideandistance> IN <store_name> WHERE (<predicate>)
```

#### Example:
```
GETSIMN 3 WITH [0.25, 0.88] USING cosinesimilarity IN my_store WHERE (category != "draft")
```

#### GET KEY
Retrieve a vector and its metadata by key.
```
GET KEY <key> IN <store_name>
```

#### Example:
```
GET KEY doc1 IN my_store
```

#### GET BY PREDICATE
Retrieve all vectors that satisfy a metadata predicate.
```
GET BY PREDICATE (<predicate>) IN <store_name>
```

#### Example:
```
GET BY PREDICATE (lang = "en") IN my_store
```


### 5. Index Management
#### CREATE PREDICATE INDEX
Create an index for faster metadata filtering.
```
CREATE PREDICATE INDEX <field> IN <store_name>
```

#### Example:
```
CREATE PREDICATE INDEX category IN my_store
```

#### DROP PREDICATE INDEX
Remove a predicate index.
```
DROP PREDICATE INDEX <field> IN <store_name>
```

#### Example:
```
DROP PREDICATE INDEX category IN my_store
```

#### CREATE NON LINEAR ALGORITHM INDEX
Build a non-linear index (e.g., KD-Tree, HNSW) for improved search efficiency.
```
CREATE NON LINEAR ALGORITHM INDEX <algorithm> IN <store_name>
```

#### Examples:
```
CREATE NON LINEAR ALGORITHM INDEX kdtree IN my_store
```
```
CREATE NON LINEAR ALGORITHM INDEX hnsw IN my_store
```

#### DROP NON LINEAR ALGORITHM INDEX
Remove a non-linear algorithm index.
```
DROP NON LINEAR ALGORITHM INDEX <algorithm> IN <store_name>
```

#### Example:
```
DROP NON LINEAR ALGORITHM INDEX kdtree IN my_store
```

## Ahnlich DB Command
This document provides a reference mapping of Ahnlich DB commands to their equivalent SDK API calls in Rust, Python, and Go.

| DB Command | Rust API Equivalent | Python API Equivalent | Go API Equivalent | 
| ----- | ----- | ----- | ----- |
| PING | `client.ping()?;` | `client.ping()` | `client.Ping(ctx)` |
| INFO SERVER | `client.info_server()?;` | `client.info_server()` | `client.InfoServer(ctx)` |
| LIST CONNECTED CLIENTS | `client.list_clients()?;` | `client.list_clients()` | `client.ListClients(ctx)` |
| LIST STORES | `client.list_stores()?;` | `client.list_stores()` | `client.ListStores(ctx)` |
| CREATE STORE my_store DIMENSION 128 ALGORITHM cosine | `client.create_store("my_store", 128, "cosine")?;` | `client.create_store("my_store", 128, "cosine")` | `client.CreateStore(ctx, "my_store", 128, "cosine")`|
| DROP STORE my_store | `client.drop_store("my_store")?;` | `client.drop_store("my_store")` | `client.DropStore(ctx, "my_store")` |
| `SET doc1 [0.25, 0.88] WITH {...} IN my_store` | `client.set("my_store", "doc1", vec![0.25,0.88], hashmap!{"category"=>"news"})?;` | `client.set("my_store", "doc1", [0.25,0.88], {"category":"news"})` | `client.Set(ctx, "my_store", "doc1", []float64{0.25,0.88}, map[string]string{"category":"news"})` |
| DELETE KEY doc1 IN my_store | `client.delete_key("my_store", "doc1")?;` | `client.delete_key("my_store", "doc1")` | `client.DeleteKey(ctx, "my_store", "doc1")` |
| DELETE PREDICATE (category = "archive") IN my_store | `client.delete_predicate("my_store", "category='archive'")?;` | `client.delete_predicate("my_store", "category='archive'")` | `client.DeletePredicate(ctx, "my_store", "category='archive'")` |
| GET SIM N 3 WITH [0.25,0.88] USING cosinesimilarity | `client.get_sim_n("my_store", vec![0.25,0.88], 3, "cosine", Some("lang='en'"))?;` | `client.get_sim_n("my_store", [0.25,0.88], 3, "cosine", predicate="lang='en'")` | `client.GetSimN(ctx, "my_store", []float64{0.25,0.88}, 3, "cosine", "lang='en'")` |
| GET KEY doc1 IN my_store | `client.get_key("my_store", "doc1")?;` | `client.get_key("my_store", "doc1")` | `client.GetKey(ctx, "my_store", "doc1")` |
| GET BY PREDICATE (lang = "en") IN my_store | `client.get_by_predicate("my_store", "lang='en'")?;` | `client.get_by_predicate("my_store", "lang='en'")` | `client.GetByPredicate(ctx, "my_store", "lang='en'")` |
| CREATE PREDICATE INDEX category IN my_store | `client.create_predicate_index("my_store", "category")?;` | `client.create_predicate_index("my_store", "category")` | `client.CreatePredicateIndex(ctx, "my_store", "category")` |
| DROP PREDICATE INDEX category IN my_store | `client.drop_predicate_index("my_store", "category")?;` | `client.drop_predicate_index("my_store", "category")` | `client.DropPredicateIndex(ctx, "my_store", "category")` |
| CREATE NON LINEAR ALGORITHM INDEX kdtree IN my_store | `client.create_non_linear_algorithm_index("my_store", NonLinearIndex { kdtree })?;` | `client.create_non_linear_algorithm_index("my_store", NonLinearIndex(kdtree=KDTreeConfig()))` | `client.CreateNonLinearAlgorithmIndex(ctx, "my_store", NonLinearIndex_Kdtree)` |
| CREATE NON LINEAR ALGORITHM INDEX hnsw IN my_store | `client.create_non_linear_algorithm_index("my_store", NonLinearIndex { hnsw })?;` | `client.create_non_linear_algorithm_index("my_store", NonLinearIndex(hnsw=HNSWConfig()))` | `client.CreateNonLinearAlgorithmIndex(ctx, "my_store", NonLinearIndex_Hnsw)` |
| DROP NON LINEAR ALGORITHM INDEX kdtree IN my_store | `client.drop_non_linear_algorithm_index("my_store", "kdtree")?;` | `client.drop_non_linear_algorithm_index("my_store", "kdtree")` | `client.DropNonLinearAlgorithmIndex(ctx, "my_store", "kdtree")` |
