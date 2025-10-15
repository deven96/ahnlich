---
title: Deeper Dive
---

# Deeper Dive

## 1. Commands via the AI Proxy
Ahnlich AI acts as a proxy service that abstracts away the complexity of generating embeddings. Instead of supplying raw vectors (as with Ahnlich DB), developers can submit natural inputs such as text, and the AI proxy will:

- Generate embeddings using the configured model(s).

- Forward the transformed embeddings to Ahnlich DB for storage or similarity queries.

### Examples of Commands
#### Ping the Server
- `PING`

Verifies that the Ahnlich AI service is running.

#### Server Info
- `INFO SERVER`

Retrieves information about the AI proxy (status, active models, connected DB).

#### List Stores
- `LIST STORES`

Returns all stores managed through the AI proxy.

#### Create a Store with Models
- `CREATESTORE my_store QUERYMODEL all-minilm-l6-v2 INDEXMODEL all-minilm-l6-v2`

This creates a store named `my_store` where:
- **index_model** → generates embeddings for stored data.

- **query_model** → generates embeddings for queries.


#### Insert Text into Store
- `SET doc1 "Renewable energy storage is key to sustainability." IN my_store`

Raw text is automatically converted into embeddings before being sent to Ahnlich DB.

#### Similarity Search with Natural Query
- `GETSIMN 3 WITH "solar battery systems" IN my_store`

The AI proxy embeds the query "`solar battery systems`", forwards it to Ahnlich DB, and retrieves the top 3 most similar entries.

#### Query by Predicate
- `GET BY PREDICATE (category = "energy") IN my_store`

Filters results using metadata conditions.

#### Create Predicate Index
- `CREATEPREDICATEINDEX category IN my_store`

Optimizes queries based on the category field.

#### Drop Predicate Index
- `DROPPREDICATEINDEX category IN my_store`

Removes an existing predicate index.

#### Create Non-Linear Algorithm Index
- `CREATENONLINEARALGORITHM INDEX kdtree IN my_store`

Enables advanced search indexing strategies (e.g., KD-Tree).

#### Drop Non-Linear Algorithm Index
- `DROPNONLINEARALGORITHMINDEX kdtree IN my_store`

Removes a non-linear algorithm index.

#### Delete by Key
- `DELETEKEY doc1 IN my_store`

Deletes a specific entry (doc1) from the store.

#### Drop Store
- `DROPSTORE my_store`

Deletes the entire store and its data.

## 2. How Ahnlich AI Reuses and Interacts with Ahnlich DB

The interaction model is two-tiered:
- **Input Transformation**

  - The AI proxy transforms raw input (e.g., "renewable energy") into a vector embedding using the configured model.

- Store Linkage

  - Each store is bound to an **index_model** (for embedding inserted data) and a **query_model** (for embedding search queries).

  - This enables dual-encoder setups where different models can be used for indexing vs. querying.

- Delegation to Ahnlich DB

  - After embedding generation, commands are translated into their Ahnlich DB equivalents.

### Example:
- `GETSIMN 3 WITH "renewable energy storage" IN article_store`

→ The AI proxy embeds the query and calls DB:
- `GETSIMN 3 WITH [0.23, 0.91, -0.44, ...] USING cosinesimilarity IN article_store`

## 3. Supported Modalities and Models

Depending on your setup, Ahnlich AI supports different modalities of input:

- **Text**

  - Embeddings generated from models like `all-minilm-l6-v2`.

  - Optimized for semantic similarity, clustering, and NLP tasks.

- **Dual Encoders (if installed)**

  - Support for cases where different models handle queries vs. indexed data.

  - Useful in retrieval systems where query understanding and corpus representation require different embedding strategies.

Important: When creating a store, you must explicitly define both:

- `CREATESTORE my_store QUERYMODEL all-minilm-l6-v2 INDEXMODEL all-minilm-l6-v2`

## 4. Advanced – Running on Non-CPU Devices

By default, **Ahnlich AI** runs on **CPU** for maximum portability.

For production-scale or latency-sensitive workloads, it can leverage specialized **execution providers** to offload embedding generation onto accelerators such as GPUs or Apple’s CoreML devices.

## Supported Execution Providers

Ahnlich AI can leverage multiple execution backends for model inference. By default, models run on **CPU execution** unless otherwise specified.

| Provider | Platform | Description |
| ----- | ----- | ----- |
| **CPU (Default)** | All platforms | Runs models on CPU by default. Portable and easy to deploy, but slower for large-scale or real-time queries. |
| **CUDA** | NVIDIA GPUs (Linux/Windows) | Runs models on CUDA-enabled GPUs. Requires `>= CUDA v12`. You may also need: `bash sudo apt install libcudnn9-dev-cuda-12` Best for batch queries or high-throughput NLP. |
| **TensorRT** | NVIDIA GPUs | NVIDIA’s optimized inference runtime. Provides lower latency than CUDA alone, especially for large models. |
| **CoreML** | macOS / iOS (M1/M2) | Apple’s ML framework for Apple Silicon.  Not advised for NLP models due to the high dimensionality of embeddings. |
| **DirectML** | Windows | Hardware-accelerated inference on Windows devices. Offers broad GPU compatibility. |

## Example – Overriding Execution Provider

By default, Ahnlich AI runs with `CPUExecutionProvider`.
You can override this when starting the engine or running a query.

### Rust Example
```
create_store(
    store="my_store",
    index_model="all-minilm-l6-v2",
    query_model="all-minilm-l6-v2",
    execution_provider="CUDA" // Override default CPU provider
)
```

## Switching Providers

- **CPU Execution (default)**: Portable and easy to deploy across environments.

- **GPU / Accelerators**: Use CUDA, TensorRT, DirectML, or CoreML for higher throughput and lower latency.
