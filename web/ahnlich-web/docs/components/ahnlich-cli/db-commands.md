---
title: DB Commands
---

# Ahnlich CLI – Database Commands

The Ahnlich CLI also supports structured database stores that allow you to insert, retrieve, and manage key-value data with predicates and indexes. Unlike AI Stores, which use embeddings and models, DB Stores are optimized for direct key-based and predicate-based queries.

A **DB Store** manages:
- **Keys and Values** – you insert plain data (strings, JSON-like objects, numbers).

- **Predicates** – metadata fields for filtering queries.

- **Indexes** – predicate and algorithmic indexes for efficient lookups.

With DB Stores, you can:

- Insert data as key-value pairs.

- Retrieve values directly by key.

- Query data using predicates.

- Create and drop indexes for better performance.

- Delete specific keys or drop entire stores.

## Example Workflow
1. **Create a DB Store** with predicates and optional indexes.

2. **Insert Data** into the store.

3. **Query Data** by key or predicate.

4. **Manage Indexes** for faster searches.

5. **Drop Stores or Keys** when they’re no longer needed.

## DB CLI Commands

Below are the most common commands you can run against your DB store:

### 1. Ping the DB server
`PING`

Checks if the DB server is alive and responding.

### 2. Get DB server information
`INFOSERVER`

Returns server metadata, including version, address, type, and resource limits.

### 3. List all DB stores
`LISTSTORES`

Lists all stores currently available on the DB server.

### 4. Create a Store for DB
```
CREATESTORE my_store PREDICATES (author, category)
```

Creates a new database store `my_store` with `author` and `category` as metadata fields.

### 5. Insert DB Data
```
SET ((key1, {author: Alice, category: ml}),
     (key2, {author: Bob, category: dev})) IN my_store
```

Inserts two records into `my_store` with associated predicates.

### 6. Drop a Store
```
DROPSTORE my_store IF EXISTS
```

Deletes the store `my_store` if it exists.

### 7. Get Data by Key
```
GET key1 IN my_store
```

Retrieves the entry with `key1` from `my_store`.

### 8. Query DB Data by Predicate
```
GETPRED (author = Alice) IN my_store
```

Retrieves all entries in `my_store` where `author = Alice`.

### 9. Create Predicate Index
```
CREATEPREDINDEX (author, category) IN my_store
```

Creates an index on `author` and `category` predicates to speed up lookups.

### 10. Drop Predicate Index
```
DROPPREDINDEX (category) IN my_store
```

Removes the index on the `category` predicate.

### 11. Create Non-Linear Algorithm Index
```
CREATENONLINEARALGORITHMINDEX (btree) IN my_store
```

Creates a B-Tree index for efficient range and predicate queries.

### 12. Drop Non-Linear Algorithm Index
```
DROPNONLINEARALGORITHMINDEX (btree) IN my_store
```

Drops the B-Tree index from `my_store`.

### 13. Delete a Key
```
DELETEKEY (key1) IN my_store
```

Deletes the entry `key1` from `my_store`.