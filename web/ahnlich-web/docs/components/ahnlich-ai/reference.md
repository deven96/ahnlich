---
title: Reference
sidebar_position: 20
---

# Reference

Ahnlich AI (ahnlich-ai) is the proxy layer that connects raw input (text, images, etc.) to **Ahnlich DB**, automatically generating embeddings and managing them in vector stores.

Below are the supported commands for **Ahnlich AI**, with examples in CLI, Rust, Python, and Go.

The supported model in your setup is:
- `all-minilm-l6-v2` (used for both indexing and querying).

## 1. Ping
#### Description
Check whether the Ahnlich AI service is up and running.

#### Command
```
PING
```

## 2. Info Server
#### Description
Returns runtime information such as version, uptime, and models available.

#### Command
```
INFOSERVER
```

## 3. List Stores
#### Description
Displays all stores currently created in Ahnlich AI.

#### Command
```
LIST STORES
```

## 4. Create Store
#### Description
Creates a new store with specified index and query models.

#### Command
```
CREATESTORE my_store INDEXMODEL all-minilm-l6-v2 QUERYMODEL all-minilm-l6-v2
```

- **Rust**
  ```
  client.create_store("my_store", "all-minilm-l6-v2", "all-minilm-l6-v2")?;
  ```

- **Python**
  ```
  client.create_store("my_store", index_model="all-minilm-l6-v2", query_model="all-minilm-l6-v2")
  ```

- **Go**
  ```
  client.CreateStore(ctx, "my_store", "all-minilm-l6-v2", "all-minilm-l6-v2")
  ```

## 5. Set
#### Description
Insert raw input into a store. Metadata can be added as key–value pairs.

#### Command
```
SET doc1 "The future of AI in healthcare" WITH {"category":"news"} IN article_store
```


## 6. Drop Store
#### Description
Removes a store and its contents permanently.

#### Command
```
DROPSTORE article_store
```

## 7. Get Sim N
#### Description
Retrieve the top N most similar vectors to a given raw input.

#### Command
```
GETSIMN 3 WITH "renewable energy storage" USING cosinesimilarity IN article_store WHERE (category != "sports")
```

## 8. Get By Predicate
#### Description
Retrieve all items in a store that satisfy a metadata condition.

#### Command
```
GETPRED (category = "news") IN article_store
```

## 9. Create Predicate Index
#### Description
Create an index on metadata to optimize predicate queries.

#### Command
```
CREATEPREDICATEINDEX category IN article_store
```

## 10. Drop Predicate Index
#### Description
Remove a previously created metadata index.

#### Command
```
DROPPREDICATEINDEX category IN article_store
```

## 11. Create Non Linear Algorithm Index
#### Description
Create an advanced index (e.g., k-d tree) for faster similarity searches.

#### Command
```
CREATENONLINEARALGORITHMINDEX kdtree IN geo_store
```

## 12. Drop Non Linear Algorithm Index
#### Description
Drop a previously created non-linear index.

#### Command
```
DROPNONLINEARALGORITHMINDEX kdtree IN geo_store
```

## 13. Delete Key
#### Description
Remove a specific key from a store.

#### Command
```
DELETEKEY doc1 IN article_store
```

## Ahnlich AI Commands

| AI Command | Rust API Equivalent | Python API Equivalent | Go API Equivalent |
| --- | --- | --- | --- |
| PING | ```client.ping()?;``` | ```client.ping()``` | ```client.Ping(ctx)``` |
| INFO SERVER | ```client.info_server()?;``` | ```client.info_server()``` | ```client.InfoServer(ctx)``` |
| LIST STORES | ```client.list_stores()?;``` | ```client.list_stores()``` | ```client.ListStores(ctx)``` |
| CREATE STORE my_store INDEXMODEL all-minilm-l6-v2 QUERYMODEL all-minilm-l6-v2 | ```client.create_store("my_store", "all-minilm-l6-v2", "all-minilm-l6-v2")?;``` | ```client.create_store("my_store", index_model="all-minilm-l6-v2", query_model="all-minilm-l6-v2")``` | ```client.CreateStore(ctx, "my_store", "all-minilm-l6-v2", "all-minilm-l6-v2")``` |
| SET doc1 "The future of AI in healthcare" WITH &#123;"category":"news"&#125; IN article_store | ```client.set("article_store", "doc1", "The future of AI in healthcare", hashmap!{"category"=>"news"})?;``` | ```client.set("article_store", "doc1", "The future of AI in healthcare", {"category":"news"})``` | ```client.Set(ctx, "article_store", "doc1", "The future of AI in healthcare", map[string]string{"category":"news"})``` |
| DROP STORE article_store | ```client.drop_store("article_store")?;``` | ```client.drop_store("article_store")``` | ```client.DropStore(ctx, "article_store")``` |
| GET SIM N 3 WITH "renewable energy storage" USING cosinesimilarity IN article_store WHERE (category != "sports") | ```client.get_sim_n("article_store", "renewable energy storage", 3, "cosine", Some("category!='sports'"))?;``` | ```client.get_sim_n("article_store", "renewable energy storage", 3, "cosine", predicate="category!='sports'")``` | ```client.GetSimN(ctx, "article_store", "renewable energy storage", 3, "cosine", "category!='sports'")``` |
| GET BY PREDICATE (category = "news") IN article_store | ```client.get_by_predicate("article_store", "category='news'")?;``` | ```client.get_by_predicate("article_store", "category='news'")``` | ```client.GetByPredicate(ctx, "article_store", "category='news'")``` |
| CREATE PREDICATE INDEX category IN article_store | ```client.create_predicate_index("article_store", "category")?;``` | ```client.create_predicate_index("article_store", "category")``` | ```client.CreatePredicateIndex(ctx, "article_store", "category")``` |
| DROP PREDICATE INDEX category IN article_store | ```client.drop_predicate_index("article_store", "category")?;``` | ```client.drop_predicate_index("article_store", "category")``` | ```client.DropPredicateIndex(ctx, "article_store", "category")``` |
| CREATE NON LINEAR ALGORITHM INDEX kdtree IN geo_store | ```client.create_non_linear_index("geo_store", "kdtree")?;``` | ```client.create_non_linear_index("geo_store", "kdtree")``` | ```client.CreateNonLinearIndex(ctx, "geo_store", "kdtree")``` |
| DROP NON LINEAR ALGORITHM INDEX kdtree IN geo_store | ```client.drop_non_linear_index("geo_store", "kdtree")?;``` | ```client.drop_non_linear_index("geo_store", "kdtree")``` | ```client.DropNonLinearIndex(ctx, "geo_store", "kdtree")``` |
| DELETE KEY doc1 IN article_store | ```client.delete_key("article_store", "doc1")?;``` | ```client.delete_key("article_store", "doc1")``` | ```client.DeleteKey(ctx, "article_store", "doc1")``` |
