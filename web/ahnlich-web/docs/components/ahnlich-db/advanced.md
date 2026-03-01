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


## 4. HNSW (Non-Linear)
**Definition:**
Hierarchical Navigable Small World (HNSW) is a graph-based approximate nearest-neighbor (ANN) search algorithm. It builds a multi-layered navigable graph that enables fast similarity search by progressively narrowing down candidates through hierarchical layers.

**When to Use:**
- High-dimensional embeddings (100+ dimensions).

- Large-scale datasets where exact search is too slow.

- Use cases where a small recall trade-off for significant speed gains is acceptable.

- Semantic search, recommendation systems, and image retrieval at scale.

**Configuration Parameters:**
HNSW performance is highly tunable via its configuration:

| **Parameter** | **Default** | **Description** |
| ----- | ----- | ----- |
| `ef_construction` | 100 | Search breadth during index building. Higher = better recall but slower inserts. |
| `maximum_connections` (M) | 48 | Max connections per node at layers > 0. Higher = more memory, better recall. |
| `maximum_connections_zero` (Mmax0) | 96 | Max connections at layer 0. Typically 2×M. |
| `extend_candidates` | false | Expand candidate pool with neighbors' neighbors during selection. |
| `keep_pruned_connections` | false | Retain pruned connections for higher connectivity (lower diversity). |
| `distance` | Euclidean | Distance metric: `Euclidean`, `Cosine`, or `DotProduct`. |

**Command to Create Index (with default config):**
```
CREATE NON LINEAR ALGORITHM INDEX hnsw IN semantic_store
```

**Command to Create Store with HNSW config:**
```python
# Python client example
db_client.create_store(
    store="semantic_store",
    dimension=384,
    create_predicates=["category"],
    non_linear_indices=[
        NonLinearIndex(index=HnswConfig(
            distance=DistanceMetric.Cosine,
            ef_construction=200,
            maximum_connections=32,
            maximum_connections_zero=64,
        ))
    ],
    error_if_exists=True,
)
```

**Query Example:**
```
GETSIMN 10 WITH [0.12, 0.45, ...] USING hnsw IN semantic_store
```

**Use Case Example:**
- A semantic search engine indexing millions of document embeddings.

- Querying "machine learning frameworks" retrieves the 10 most semantically similar documents in sub-millisecond time.

**Tuning Tips:**
- **Low recall?** Increase `ef_construction` and `maximum_connections` to build a denser graph.
- **Slow inserts?** Decrease `ef_construction` for faster index building at the cost of recall.
- **Memory constrained?** Lower `maximum_connections` to reduce memory footprint.
- **Reconstruction:** If you created an index with poor config, you can drop it and recreate with better values. The existing data will be re-indexed automatically.


## Choosing the Right Algorithm

| **Algorithm** | **Best For** | **Pros** | **Cons** |
| ----- | ----- | ----- | ----- |
| Cosine Similarity | NLP, semantic search | Ignores magnitude, fast | Not good for magnitude-based data |
| Euclidean Distance | Images, structured numeric features | Intuitive, works with magnitude | Slower in very high dimensions |
| k-d Tree | Geospatial, medium-dim embeddings | Efficient exact NN search | Struggles in very high dimensions |
| HNSW | High-dim, large-scale datasets | Fast ANN search, tunable recall/speed | Approximate (not exact), higher memory |


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


#### 4. HNSW (Non-Linear Index)
- **Speed:** Sub-millisecond for most queries, even on large datasets.

- **Accuracy:** Approximate, but tunable via configuration parameters.

- **Memory Usage:** Higher than linear algorithms due to graph structure.

**Benchmark Insight (example):**
- Dataset: 10K SIFT vectors (128-dim).

- Avg. Query Latency: <1ms (graph traversal).

- Accuracy: 90%+ recall@50 with default config; higher with tuned parameters.

**Limitation:** Approximate results (not exact). Quality depends on configuration.

**Best Trade-off:**
- Ideal for **high-dimensional semantic search at scale** where speed matters more than perfect accuracy.


**Summary Table**

| **Algorithm** | **Speed** | **Accuracy** | **Best Use Case** | **Weakness** |
| ----- | ----- | ----- | ----- | ----- |
| Cosine Similarity | Fast | High (95%) | Semantic search (NLP, docs) | Ignores magnitude differences |
| Euclidean Distance | Moderate | High (93%) | Image search, recommendations | Slower in high dims |
| k-d Tree | Ultra-Fast (low-dim) | Exact (100%) | Geospatial, low-dim structured data | Weak in high-dimensional space |
| HNSW | Ultra-Fast (any dim) | Tunable (80-99%) | Large-scale high-dim search | Approximate, higher memory |


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
Returns the vector stores available in the DB, including store name, entry count, size in bytes, and the configuration of any non-linear indices (HNSW or k-d tree) on each store.


- **CREATE STORE** `<name>`
Creates a new container for vectors + metadata. Optionally accepts non-linear index configurations (HNSW with tunable parameters, or k-d tree).
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

- Supports linear (`cosine, euclidean`) and non-linear (`kdtree, hnsw`).

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
 Builds a non-linear index (k-d tree or HNSW) for nearest-neighbor queries.
```
> CREATE NON LINEAR ALGORITHM INDEX kdtree ON geodata
> CREATE NON LINEAR ALGORITHM INDEX hnsw ON semantic_store
```

- **DROP NON LINEAR ALGORITHM INDEX**
 Removes a non-linear index (k-d tree or HNSW).



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

**HNSW Index**
- Builds a **hierarchical graph structure** for approximate nearest-neighbor search.

- Designed for **high-dimensional data at scale** (100+ dimensions, millions of vectors).

- Configurable trade-off between recall accuracy and search speed.

- Supports Euclidean, Cosine, and DotProduct distance metrics.

```
> CREATE NON LINEAR ALGORITHM INDEX hnsw ON semantic_store
> GETSIMN 10 WITH [0.12, 0.45, 0.78, ...] USING hnsw IN semantic_store
```

**Example use case:** Search through millions of document embeddings for the most semantically similar content in sub-millisecond time.
 Note: Results are approximate. Tune `ef_construction` and `maximum_connections` for better recall.

### 3. Choosing the Right Algorithm
| **Algorithm** | **Strengths** | **Weaknesses** | **Example Use Case** |
| ----- | ----- | ----- | ----- |
| Cosine Similarity | Fast, ignores scale, semantic focus | Magnitude differences ignored | NLP semantic search |
| Euclidean Distance | Captures true closeness, magnitude | Slower in high dims | Image search, recommendations |
| k-d Tree | Exact, blazing fast in low-dim | Poor in >50 dimensions | Geospatial queries |
| HNSW | Fast ANN, works in high dims, tunable | Approximate, uses more memory | Large-scale semantic search |


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