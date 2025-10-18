---
title: AI Commands
---

# Ahnlich CLI – AI Commands

The Ahnlich CLI supports **AI-powered vector stores** that enable semantic search, similarity matching, and predicate-based queries. This allows developers to insert embeddings (from text, images, or other binary inputs), query them with different similarity algorithms, and manage AI stores just like databases.

An **AI Store** is a specialized store that maintains:

- **Query Model** – model used to process incoming query inputs (e.g., `resnet-50`, `all-MiniLM-L6-v2`).

- **Index Model** – model used to generate embeddings for inserted data.

- **Predicates** – metadata fields associated with each input (e.g., `author`, `category`).

- **Non-Linear Algorithm Index** – optional advanced indexing (e.g., `kdtree`) to accelerate nearest-neighbor search.

With AI Stores, you can:

- Insert text, image, or binary inputs with metadata.

- Run similarity searches (`cosinesimilarity`, `l2`, etc.).

- Filter results by predicates.

- Create and manage indexes for faster queries.

- Delete stores or individual keys when no longer needed.

## Example Workflow

1. **Create an AI Store** with models and metadata fields.

2. **Insert AI Data** (text or image embeddings + metadata).

3. **Query AI Data** using similarity search.

4. **Refine queries** with predicates and indexes.

5. **Manage lifecycle** of stores, indexes, and entries.

## AI CLI Commands

Below are the most common commands you can run against your AI store:

### 1. Ping the AI server
`PING`

Checks if the AI server is alive and responding.

### 2. Get AI server information
`INFOSERVER`

Returns server metadata, including version, address, type, and resource limits.

### 3. List all AI stores
`LISTSTORES`

Lists all stores currently available on the AI server.

### 4. Create a Store for AI
```
CREATESTORE my_store QUERYMODEL resnet-50 INDEXMODEL resnet-50 PREDICATES (author, category) NONLINEARALGORITHMINDEX (kdtree) STOREORIGINAL
```

Creates a new store `my_store` with `resnet-50` as both query and index models, supporting predicates `author` and `category`, and enables a KD-Tree index.

### 5. Insert AI Data
```
SET (([This is the life of Alice], {author: Alice, category: ml}),
     ([This is the life of Bob], {author: Bob, category: dev})) IN my_store
```

Inserts two text entries into `my_store` with metadata tags.

### 6. Drop a Store
```
DROPSTORE my_store IF EXISTS
```

Deletes the store `my_store` if it exists.

### 7. Query AI Data by Similarity
```
GETSIMN 4 WITH [This is the life of Alice] USING cosinesimilarity IN my_store WHERE (category = ml)
```

Finds the top 4 entries most similar to `"This is the life of Alice"` within category `ml`.

### 8. Query AI Data by Predicate
```
GETPRED (author = Alice) IN my_store
```

Retrieves all entries in `my_store` where `author = Alice`.

### 9. Create Predicate Index
```
CREATEPREDINDEX (author, category) IN my_store
```

Creates an index on the `author` and `category` predicates to speed up lookups.

### 10. Drop Predicate Index
```
DROPPREDINDEX (category) IN my_store
```

Removes the index on the `category` predicate.

### 11. Create Non-Linear Algorithm Index
```
CREATENONLINEARALGORITHMINDEX (kdtree) IN my_store
```

Creates a KD-Tree index for non-linear similarity search.

### 12. Drop Non-Linear Algorithm Index
```
DROPNONLINEARALGORITHMINDEX (kdtree) IN my_store
```

Drops the KD-Tree index from the store.

### 13. Delete a Key
```
DELETEKEY ([This is the life of Alice]) IN my_store
```

Deletes the entry `"This is the life of Alice"` from `my_store`.