---
title: 🗄️ Ahnlich DB
sidebar_position: 10
---

# Ahnlich DB

## Overview

**Ahnlich DB** is the core storage and retrieval engine of the Ahnlich ecosystem. It is an **in-memory vector key–value store** that enables developers, data scientists, and ML engineers to persist embeddings (high-dimensional numerical representations) alongside descriptive metadata. This combination allows for powerful and flexible similarity search across large sets of data.

Unlike traditional databases that store rows and columns of scalar values, **Ahnlich DB** is optimized for storing **vector representations** of data commonly produced by machine learning models such as those for natural language processing, computer vision, or recommendation systems. Each vector is stored with **metadata**, making it possible to not only find the most similar vectors but also to **filter results** based on attributes.

**Ahnlich DB** serves as the foundation of the platform. By storing embeddings and enabling efficient similarity queries, it supports higher-level Ahnlich components such as AI pipelines, application SDKs, and domain-specific tools.

In essence, **Ahnlich DB** **is the backbone of vector storage and retrieval in Ahnlich**, providing both performance and flexibility for modern AI applications.


### 1. Vector Similarity Search
**Ahnlich DB** supports **nearest-neighbor queries**, enabling you to retrieve vectors that are closest to a given input embedding. This is essential for tasks like semantic search, recommendations, or clustering.
- **Linear Methods:**

**Cosine Similarity** – measures the angle between two vectors, useful when magnitude doesn’t matter (e.g., comparing text embeddings).

**Euclidean Distance** – measures geometric distance, useful when absolute differences are important (e.g., image embeddings).


- **Non-linear Methods:**

**k-d Tree** – partitions vectors into a hierarchical structure for faster searches in high-dimensional space.


**Example:**
 A user searches for articles similar to “renewable energy storage.” The text is converted into a vector and matched against stored article embeddings.

```go
GETSIMN 3 WITH [0.23, 0.91, -0.44, ...] USING cosinesimilarity 
IN article_store WHERE (category != "sports")
```

This returns the top 3 most semantically similar articles, excluding those in the sports category.

### 2. Metadata Filtering
Every stored vector can be annotated with **key–value metadata**, allowing you to combine similarity with structured filtering. This ensures results are not just “close” but also **contextually valid**.

**Example:**
 A music app wants to recommend similar songs but only from the Jazz genre.

```go
GETSIMN 5 WITH [0.11, 0.75, -0.32, ...] USING euclideandistance 
IN music_store WHERE (genre = "jazz")
```

This retrieves the top 5 songs similar to the input vector, but only those labeled with `genre = jazz`.

### 3. In-Memory Performance
**Ahnlich DB** is designed to be **in-memory**, making queries extremely fast. This is especially valuable for **real-time AI/ML workflows** like personalized recommendations, chatbots, or anomaly detection, where latency must be minimal.

**Example:**
 During a live chat, a customer support assistant uses embeddings of past tickets to suggest possible solutions instantly. **Ahnlich DB** runs these similarity lookups in-memory, ensuring sub-second responses.

### 4. Flexible Querying
**Ahnlich DB** can be queried by:
**API-based programmatic access** (Rust, Python, or SDKs).


This flexibility allows **Ahnlich DB** to integrate into different pipelines—whether experimental research notebooks or production-grade AI services.

**Example:**
**Command Interface:**
```go
GETSIMN 10 WITH [0.55, 0.82, -0.17, ...] USING cosinesimilarity 
IN user_store WHERE (status = "active")
```

Rust API:
```go
get_sim_n(
    store="user_store",
    search_input=[0.55, 0.82, -0.17, ...],
    closest_n=10,
    algorithm=CosineSimilarity,
    condition=Predicate::Equals {
        key="status",
        value="active",
    },
)
```

Both queries return the 10 most similar active users.

