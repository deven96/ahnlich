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
Displays AI stores in the selected schema. If no schema is supplied, only `public` stores are returned.

#### Command
```
LISTSTORES
LISTSTORES SCHEMA media
```

## 4. Get Store
#### Description
Get detailed information about a specific store by name. Returns store name, query model, index model, embedding size, and optionally the underlying DB store info (when AI is connected to a DB instance). The `db_info` field contains the DB store's name, length, size in bytes, non-linear indices, predicate indices, and dimension. Returns an error if the store does not exist.

#### Command
```
GETSTORE my_store
GETSTORE my_store SCHEMA media
```

## 5. Create Store
#### Description
Creates a new store with specified index and query models.

#### Command
```
CREATESTORE my_store QUERYMODEL all-minilm-l6-v2 INDEXMODEL all-minilm-l6-v2
CREATESTORE my_store QUERYMODEL all-minilm-l6-v2 INDEXMODEL all-minilm-l6-v2 SCHEMA media
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

## 6. Set
#### Description
Insert raw input into a store. Metadata can be added as key–value pairs.

#### Command
```
SET doc1 "The future of AI in healthcare" WITH {"category":"news"} IN article_store
SET doc1 "The future of AI in healthcare" WITH {"category":"news"} IN article_store SCHEMA media
```


## 7. Drop Store
#### Description
Removes a store and its contents permanently.

#### Command
```
DROPSTORE article_store
DROPSTORE article_store IF EXISTS SCHEMA media
```

## 8. Drop Schema
#### Description
Drops a non-public schema and all AI stores inside it. Ahnlich AI also drops the backing DB schema before removing its local AI stores. The `public` schema cannot be dropped.

#### Command
```
DROPSCHEMA media
```

## 9. Get Sim N
#### Description
Retrieve the top N most similar vectors to a given raw input.

#### Command
```
GETSIMN 3 WITH [renewable energy storage] USING cosinesimilarity IN article_store WHERE (category != sports)
GETSIMN 3 WITH [renewable energy storage] USING cosinesimilarity IN article_store WHERE (category != sports)
GETSIMN 3 WITH [renewable energy storage] USING cosinesimilarity IN article_store SCHEMA media WHERE (category != sports)
```

## 10. Get By Predicate
#### Description
Retrieve all items in a store that satisfy a metadata condition.

#### Command
```
GETPRED (category = "news") IN article_store
GETPRED (category = "news") IN article_store SCHEMA media
```

## 11. Get Key
#### Description
Retrieve items from a store by their original input key when the store preserves originals.

#### Command
```
GETKEY doc1 IN article_store
GETKEY doc1 IN article_store SCHEMA media
```

## 12. Create Predicate Index
#### Description
Create an index on metadata to optimize predicate queries.

#### Command
```
CREATEPREDICATEINDEX category IN article_store
CREATEPREDICATEINDEX category IN article_store SCHEMA media
```

## 13. Drop Predicate Index
#### Description
Remove a previously created metadata index.

#### Command
```
DROPPREDICATEINDEX category IN article_store
DROPPREDICATEINDEX category IN article_store SCHEMA media
```

## 14. Create Non Linear Algorithm Index
#### Description
Create an advanced index (e.g., KDTree, HNSW) for faster similarity searches.

#### Command
```
CREATENONLINEARALGORITHMINDEX (kdtree) IN geo_store
CREATENONLINEARALGORITHMINDEX (hnsw) IN geo_store
CREATENONLINEARALGORITHMINDEX (kdtree) IN geo_store SCHEMA media
CREATENONLINEARALGORITHMINDEX (hnsw) IN geo_store
CREATENONLINEARALGORITHMINDEX (hnsw) IN geo_store SCHEMA media
```

## 15. Drop Non Linear Algorithm Index
#### Description
Drop a previously created non-linear index.

#### Command
```
DROPNONLINEARALGORITHMINDEX (kdtree) IN geo_store
DROPNONLINEARALGORITHMINDEX (kdtree) IN geo_store
DROPNONLINEARALGORITHMINDEX (kdtree) IN geo_store SCHEMA media
```

## 16. Upsert
#### Description
Update a single entry matching a predicate condition. Always merges metadata (preserves AI-generated fields). Errors if 0 or multiple entries match.

#### Command
```
UPSERT VALUE {tags: cat,outdoors} IN images WHERE (filename = photo.jpg)
UPSERT KEY [updated image bytes] IN images WHERE (id = 42) PREPROCESSACTION modelpreprocessing
UPSERT KEY [new text] VALUE {author: Jane} IN docs WHERE (id = 100)
UPSERT VALUE {tags: cat,outdoors} IN images SCHEMA media WHERE (filename = photo.jpg)
```

Note: AI proxy automatically preserves AI-generated metadata when merging.

## 17. Delete Key
#### Description
Remove a specific key from a store.

#### Command
```
DELETEKEY doc1 IN article_store
DELETEKEY doc1 IN article_store SCHEMA media
```

## Ahnlich AI Commands

| AI Command | Rust API Equivalent | Python API Equivalent | Go API Equivalent |
| --- | --- | --- | --- |
| PING | ```client.ping()?;``` | ```client.ping()``` | ```client.Ping(ctx)``` |
| INFO SERVER | ```client.info_server()?;``` | ```client.info_server()``` | ```client.InfoServer(ctx)``` |
| LIST STORES | ```client.list_stores()?;``` | ```client.list_stores()``` | ```client.ListStores(ctx)``` |
| GETSTORE my_store | ```client.get_store("my_store")?;``` | ```client.get_store("my_store")``` | ```client.GetStore(ctx, "my_store")``` |
| CREATE STORE my_store INDEXMODEL all-minilm-l6-v2 QUERYMODEL all-minilm-l6-v2 | ```client.create_store("my_store", "all-minilm-l6-v2", "all-minilm-l6-v2")?;``` | ```client.create_store("my_store", index_model="all-minilm-l6-v2", query_model="all-minilm-l6-v2")``` | ```client.CreateStore(ctx, "my_store", "all-minilm-l6-v2", "all-minilm-l6-v2")``` |
| SET doc1 "The future of AI in healthcare" WITH &#123;"category":"news"&#125; IN article_store | ```client.set("article_store", "doc1", "The future of AI in healthcare", hashmap!{"category"=>"news"})?;``` | ```client.set("article_store", "doc1", "The future of AI in healthcare", {"category":"news"})``` | ```client.Set(ctx, "article_store", "doc1", "The future of AI in healthcare", map[string]string{"category":"news"})``` |
| DROP STORE article_store | ```client.drop_store("article_store")?;``` | ```client.drop_store("article_store")``` | ```client.DropStore(ctx, "article_store")``` |
| DROPSCHEMA media | ```grpc_client.drop_schema(DropSchema { schema: "media".into() }).await?;``` | ```await client.drop_schema(DropSchema(schema="media"))``` | ```client.DropSchema(ctx, &query.DropSchema{Schema: "media"})``` |
| GET SIM N 3 WITH "renewable energy storage" USING cosinesimilarity IN article_store WHERE (category != "sports") | ```client.get_sim_n("article_store", "renewable energy storage", 3, "cosine", Some("category!='sports'"))?;``` | ```client.get_sim_n("article_store", "renewable energy storage", 3, "cosine", predicate="category!='sports'")``` | ```client.GetSimN(ctx, "article_store", "renewable energy storage", 3, "cosine", "category!='sports'")``` |
| GET BY PREDICATE (category = "news") IN article_store | ```client.get_by_predicate("article_store", "category='news'")?;``` | ```client.get_by_predicate("article_store", "category='news'")``` | ```client.GetByPredicate(ctx, "article_store", "category='news'")``` |
| GET KEY doc1 IN article_store | ```client.get_key(GetKey { store: "article_store".into(), keys: vec![StoreInput { value: Some(store_input::Value::RawString("doc1".into())) }], schema: None }, None).await?;``` | ```await client.get_key(GetKey(store="article_store", keys=[StoreInput(raw_string="doc1")], schema=None))``` | ```client.GetKey(ctx, &query.GetKey{Store: "article_store", Keys: []*keyval.StoreInput{&keyval.StoreInput{Value: &keyval.StoreInput_RawString{RawString: "doc1"}}}})``` |
| CREATE PREDICATE INDEX category IN article_store | ```client.create_predicate_index("article_store", "category")?;``` | ```client.create_predicate_index("article_store", "category")``` | ```client.CreatePredicateIndex(ctx, "article_store", "category")``` |
| DROP PREDICATE INDEX category IN article_store | ```client.drop_predicate_index("article_store", "category")?;``` | ```client.drop_predicate_index("article_store", "category")``` | ```client.DropPredicateIndex(ctx, "article_store", "category")``` |
| CREATE NON LINEAR ALGORITHM INDEX kdtree IN geo_store | ```client.create_non_linear_algorithm_index("geo_store", NonLinearIndex { kdtree })?;``` | ```client.create_non_linear_algorithm_index("geo_store", NonLinearIndex(kdtree=KDTreeConfig()))``` | ```client.CreateNonLinearAlgorithmIndex(ctx, "geo_store", NonLinearIndex_Kdtree)``` |
| CREATE NON LINEAR ALGORITHM INDEX hnsw IN geo_store | ```client.create_non_linear_algorithm_index("geo_store", NonLinearIndex { hnsw })?;``` | ```client.create_non_linear_algorithm_index("geo_store", NonLinearIndex(hnsw=HNSWConfig()))``` | ```client.CreateNonLinearAlgorithmIndex(ctx, "geo_store", NonLinearIndex_Hnsw)``` |
| DROP NON LINEAR ALGORITHM INDEX kdtree IN geo_store | ```client.drop_non_linear_algorithm_index("geo_store", "kdtree")?;``` | ```client.drop_non_linear_algorithm_index("geo_store", "kdtree")``` | ```client.DropNonLinearAlgorithmIndex(ctx, "geo_store", "kdtree")``` |
| DELETE KEY doc1 IN article_store | ```client.delete_key("article_store", "doc1")?;``` | ```client.delete_key("article_store", "doc1")``` | ```client.DeleteKey(ctx, "article_store", "doc1")``` |
