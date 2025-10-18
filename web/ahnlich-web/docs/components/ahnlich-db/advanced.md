---
title: Advanced
sidebar_position: 30
---

# Advanced

Ahnlich DB supports multiple similarity algorithms that determine how vectors are compared. Choosing the right one depends on your use case (recommendations, semantic search, image retrieval, etc.).

## 1. Cosine Similarity (Linear)
**Definition:**
Measures the cosine of the angle between two vectors. It focuses on the **orientation** rather than magnitude.

**When to Use:**
- Text embeddings (semantic similarity, NLP).

- High-dimensional embeddings where magnitude isn’t important.

**Example Command:**
```
GETSIMN 3 WITH [0.25, 0.88] USING cosinesimilarity IN my_store
``` 

**Use Case Example:**
- Query: "What’s the capital of France?"

- Store contains embeddings for documents about Paris, Rome, and Madrid.

- Cosine similarity retrieves the document most semantically aligned with "Paris".


## 2. Euclidean Distance (Linear)
**Definition:**
Measures the straight-line (L2 norm) distance between two vectors.

**When to Use:**
- Image embeddings (where vector magnitude carries meaning).

-Recommendation engines where closeness in feature space is meaningful.


**Example Command:**
```
GETSIMN 5 WITH [0.12, 0.45] USING euclidean IN image_store
```
**Use Case Example:**
- Searching for similar product images in an e-commerce system.

- A handbag photo returns visually similar handbags.


## 3. k-d Tree (Non-Linear)
**Definition:**
 A space-partitioning data structure for organizing vectors in **k-dimensional space**.
 It allows efficient nearest-neighbor search for structured, spatially distributed vectors.

**When to Use:**
- Lower-dimensional embeddings (≤ 50 dimensions).

- Spatial/geometric queries (e.g., geolocation vectors).


- When you need **fast exact search** over medium-sized datasets.


**Command to Create Index:**
```
CREATE NON LINEAR ALGORITHM INDEX kdtree IN geo_store
```

**Query Example:**
```
GETSIMN 10 WITH [40.7128, -74.0060] USING kdtree IN geo_store
```

**Use Case Example:**
- A ride-sharing app storing city coordinates.

- Querying with New York City coordinates retrieves the 10 nearest drivers.


## Choosing the Right Algorithm

| **Algorithm** | **Best For** | **Pros** | **Cons** | 
| ----- | ----- | ----- | ----- |
| Cosine Similarity | NLP, semantic search | Ignores magnitude, fast | Not good for magnitude-based data |
| Euclidean Distance | Images, structured numeric features | Intuitive, works with magnitude | Slower in very high dimensions |
| k-d Tree | Geospatial, medium-dim embeddings | Efficient exact NN search | Struggles in very high dimensions |


### Performance & Trade-offs of Similarity Algorithms in Ahnlich DB
Ahnlich DB is optimized for **real-time vector similarity search**, but different algorithms behave differently depending on **data size, dimensionality, and query type**. Below is a detailed comparison.

#### 1. Cosine Similarity
- **Speed:**  Very fast (linear scan over vectors).

- **Accuracy:** High for semantic embeddings.

- **Memory Usage:** Moderate, since normalization is required.

**Benchmark Insight (example):**
- Dataset: 1M text embeddings (768-dim BERT).

- Avg. Query Latency: ~15ms (on 16-core CPU, in-memory).

- Accuracy: 95% recall@10 compared to brute-force.

**Best Trade-off:**
- Works best in high-dimensional semantic spaces (text, NLP).


#### 2. Euclidean Distance
- **Speed:**  Similar to cosine, but distance calculations are slightly heavier.

- **Accuracy:** High when magnitude matters.

- **Memory Usage:** Higher than cosine if embeddings are not normalized.


**Benchmark Insight (example):**
- Dataset: 5M product images (512-dim CLIP embeddings).

- Avg. Query Latency: ~25ms.

- Accuracy: 93% recall@10.

**Best Trade-off:**
- Ideal for **recommendations, computer vision, and structured numeric data**.


#### 3. k-d Tree (Non-Linear Index)
<!-- - **Speed:**  Extremely fast for **low-dimensional data (<50 dims)**. -->

- **Accuracy:** Exact nearest-neighbor search (100%).

- **Memory Usage:** Higher (tree indexing overhead).

**Benchmark Insight (example):**
- Dataset: 10M geolocation vectors (2D lat-long).

- Avg. Query Latency: ~3ms (index lookup).

- Accuracy: 100% exact matches.


**Limitation:** Performance drops for very high-dimensional data (curse of dimensionality).

**Best Trade-off:**
- Perfect for **geospatial queries, structured 2D/3D data**.


**Summary Table**

| **Algorithm** | **Speed** | **Accuracy** | **Best Use Case** | **Weakness** |
| ----- | ----- | ----- | ----- | ----- |
| Cosine Similarity | Fast | High (95%) | Semantic search (NLP, docs) | Ignores magnitude differences |
| Euclidean Distance | Moderate | High (93%) | Image search, recommendations | Slower in high dims |
| k-d Tree | Ultra-Fast (low-dim) | Exact (100%) | Geospatial, low-dim structured data | Weak in high-dimensional space |


## Ahnlich DB – Deeper Dive into Commands & Advanced Similarity
This section explores how to **control the database through commands**, how queries are executed, and how similarity algorithms behave in practice. Think of it as a **power user’s manual** for working with Ahnlich DB.

### 1. Command Deep Dive
#### 1.1 Server Management
**PING**
 Tests if the DB is alive. Essential for monitoring tools.

`> PING`
`< PONG`

- **INFO SERVER**
 Returns server metadata: version, uptime, active stores, tracing status.

```
> INFO SERVER
< {"version":"0.0.2","uptime":"3h45m","stores":["docs","images"]}
```

- **LIST CONNECTED CLIENTS**
 Shows all clients with their IP and connection status. Useful for debugging distributed workloads.


#### 1.2 Store Lifecycle
- **LIST STORES** 
Returns the vector stores available in the DB.


- **CREATE STORE** `<name>`
Creates a new container for vectors + metadata.
```
> CREATE STORE articles
< OK
```

- **DROP STORE** `<name>`
Deletes a store permanently. Data cannot be recovered unless persistence is enabled.


#### 1.3 Vector Operations
- **SET**
 Insert or overwrite a vector embedding + metadata.
```
 > SET doc1 [0.12, 0.33, 0.44] WITH {"topic":"ai","visibility":"public"}
< OK
```

- **GET KEY**
 Retrieve a vector and its metadata by ID.
```
 > GET KEY doc1
< {"vector":[0.12,0.33,0.44],"metadata":{"topic":"ai","visibility":"public"}}
```

- **DELETE KEY**
 Remove a vector completely.


#### 1.4 Querying & Filtering
- **GET SIM N**
Core similarity search query. Finds the N closest vectors to an input.

- Supports linear (`cosine, euclidean`) and non-linear (`kdtree`).

- Can apply metadata filters.

```
> GETSIMN 3 WITH [0.2,0.1,0.7] USING cosinesimilarity IN articles WHERE (visibility = "public")
< [{"key":"doc5","score":0.92},{"key":"doc3","score":0.89},{"key":"doc7","score":0.87}]
```

- **GET BY PREDICATE**
 Filter based on metadata conditions without running similarity search.
```
> GET BY PREDICATE topic = "ai" IN articles
```

- **DELETE PREDICATE**
 Bulk delete vectors based on metadata.
```
> DELETE PREDICATE visibility = "hidden" IN articles
```

#### 1.5 Indexes
Indexes allow Ahnlich DB to optimize lookups.

- **CREATE PREDICATE INDEX**
 Speeds up metadata filtering.
```
> CREATE PREDICATE INDEX ON articles(topic)
```
- **DROP PREDICATE INDEX**
 Remove an index if it’s not needed.


- **CREATE NON LINEAR ALGORITHM INDEX**
 Builds a k-d tree index for non-linear nearest-neighbor queries.
```
> CREATE NON LINEAR ALGORITHM INDEX kdtree ON geodata
```

- **DROP NON LINEAR ALGORITHM INDEX**
 Removes the k-d tree.



### 2. Advanced Similarity Algorithms
Ahnlich DB supports both **linear** and **non-linear** similarity searches, giving engineers flexibility depending on data type, size, and query needs.

#### Linear Algorithms
**Cosine Similarity**
- Measures the **angle** between two vectors.

- Magnitude is ignored → perfect for **semantic embeddings** (e.g., text/doc search).

- Lightweight, fast to compute.

```
get_sim_n(
    store="articles",
    search_input=[0.21, 0.45, 0.76],
    closest_n=5,
    algorithm=CosineSimilarity,
    condition=Predicate::Equals{ key="visibility", value="public" },
)
```

**Example use case:** Search for documents most semantically related to a query sentence, ignoring length differences.

**Euclidean Distance**
- Measures **absolute distance** in vector space.

- Magnitude matters → good for **images, recommendations, sensor data**.

- More expensive than cosine but captures real “closeness.”

```
db.get_sim_n(
    store="images",
    search_input=[0.12, 0.33, 0.87],
    closest_n=10,
    algorithm="euclidean",
)
```

**Example use case:** Finding visually similar product images where scale and brightness differences matter.

#### Non-Linear Algorithms
**k-d Tree Index**
- Builds a **space-partitioning structure** for fast nearest-neighbor lookup.

- Perfect for **low-dimensional structured data (2D, 3D, 50 dims)**.

- Exact, not approximate.

```
> CREATE NON LINEAR ALGORITHM INDEX kdtree ON geodata
> GETSIMN 5 WITH [40.71,-74.00] USING kdtree IN geodata
```

**Example use case:** Find the 5 closest stores to a user’s GPS location.
 Note: Performance drops in high dimensions (curse of dimensionality).

### 3. Choosing the Right Algorithm
| **Algorithm** | **Strengths** | **Weaknesses** | **Example Use Case** | 
| ----- | ----- | ----- | ----- |
| Cosine Similarity | Fast, ignores scale, semantic focus | Magnitude differences ignored | NLP semantic search |
| Euclidean Distance | Captures true closeness, magnitude | Slower in high dims | Image search, recommendations | 
| k-d Tree | Exact, blazing fast in low-dim | Poor in >50 dimensions | Geospatial queries |


### 4. Putting It Together – End-to-End Flow
- CREATE STORE
```
CREATE STORE articles
```

- INSERT DATA
```
SET doc1 [0.12,0.33,0.44] WITH {"topic":"ai","visibility":"public"}
SET doc2 [0.50,0.61,0.11] WITH {"topic":"finance","visibility":"public"}
```

- BUILD INDEXES
```
CREATE PREDICATE INDEX ON articles(topic)
```

- RUN QUERIES
```
GETSIMN 3 WITH [0.20,0.10,0.70] USING cosinesimilarity IN articles WHERE (topic="ai")
```