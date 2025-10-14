---
title: 🤖 Ahnlich AI
sidebar_position: 30
---

# Ahnlich AI

## Overview

Ahnlich AI (`ahnlich-ai`) is the **AI proxy layer** for Ahnlich DB, designed to make working with embeddings **effortless and intuitive**. While Ahnlich DB specializes in fast, in-memory storage and similarity search of vector embeddings, it expects developers to provide embeddings themselves. Ahnlich AI solves this problem by **handling the embedding generation pipeline automatically**.

Instead of manually computing vectors with external libraries and then inserting them into the database, developers can work with **raw inputs** such as text, images, or other modalities. Ahnlich AI transforms those inputs into embeddings using **off-the-shelf machine learning models**, stores them into the right vector store, and later applies the same transformation logic when queries are made.

This design allows developers and engineers to focus on solving **application-level problems** such as building semantic search, recommendation engines, multimodal systems, or intelligent assistants without worrying about the complexity of embedding generation, model integration, or consistency between queries and stored data.

### Ahnlich AI introduces the concept of model-aware stores.
When creating a store, you must specify:
- **Index Model** – used when inserting new data into the store. Each input (text, image, etc.) is transformed into a vector embedding with this model before being stored.

- **Query Model** – used when searching the store. Each query input is transformed with this model to ensure results are compared in the same semantic space.

- **Constraint** – both the index model and the query model must produce embeddings of the **same dimensionality**. This ensures compatibility between stored vectors and query vectors, allowing accurate similarity comparisons.

This separation allows you to configure workflows for **different modalities** or even **cross-modal retrieval**. For example:

This separation allows you to configure workflows for different use cases while keeping embeddings aligned in the same dimensional space. For example:
- A store that indexes and queries with **all-minilm-l6-v2** for semantic text search.

- A store that uses two compatible models with the **same embedding dimensions**, where one is optimized for indexing documents and the other for handling queries.

### Example – Creating a Store:

```
CREATESTORE my_store QUERYMODEL all-minilm-l6-v2 INDEXMODEL all-minilm-l6-v2
```

Rust API equivalent:

``` rust
  create_store(
      store="my_store",
      index_model="all-minilm-l6-v2",
      query_model="all-minilm-l6-v2",
  )
```

This ensures that whenever you insert or query `my_store`, Ahnlich AI automatically applies the right embedding model under the hood.

## 1. Raw Input to Embeddings

Traditionally, developers relied on external libraries (e.g., PyTorch, HuggingFace) to generate embeddings before pushing them into a database. With Ahnlich AI, this step can be handled directly within the proxy using off-the-shelf models, simplifying the workflow.

### Text Input Example:

```
INSERT "The rise of renewable energy storage solutions" INTO article_store
```

- Ahnlich AI transforms the sentence into a vector embedding (e.g., [0.12, -0.34, 0.91, ...]), then sends it to Ahnlich DB for storage.

## 2. Model-Aware Stores
Because embeddings depend heavily on which model is used, Ahnlich AI makes stores **model-aware**.

- If a store is configured with a **text model**, both data and queries are handled as text embeddings.

- If configured with a **text-to-image setup**, one model handles indexing images and another handles queries in text.


This flexibility allows **multimodal workflows**, where developers don’t need to manually align embeddings.

### Unimodal Example (Text-to-Text)

```
CREATESTORE product_store QUERYMODEL all-minilm-l6-v2 INDEXMODEL all-minilm-l6-v2
```

- At insert time, product data (e.g., descriptions, attributes) is embedded using the index model (`all-minilm-l6-v2`).

- At query time, natural text inputs like *“blue denim jacket”* are also embedded using the query model (`all-minilm-l6-v2`).

- Since both use the same model and dimensions, embeddings exist in the same semantic space, making similarity search possible.

### Cross-Modal Example (Text-to-Image)

```
CREATESTORE image_store QUERYMODEL all-minilm-l6-v2 INDEXMODEL resnet-50
```

- At insert time, product images are embedded using the index model (`resnet-50`).

- At query time, text queries like *“blue denim jacket”* are embedded using the query model (`all-minilm-l6-v2`).

- Because Ahnlich aligns embeddings across modalities, the system can retrieve images relevant to the text query.

This ensures that Ahnlich AI can support both **unimodal (text-text, image-image)** and **cross-modal (text-image, image-text)** scenarios effectively.

## 3. Off-the-Shelf Models

Ahnlich AI comes with several **pre-integrated models** that work out of the box for text and image similarity tasks.

Instead of worrying about installation, configuration, or fine-tuning, developers simply declare which models to use when creating a store.

See the **Supported Models** section for the full list of available text and image models, including MiniLM, BGE variants, ResNet-50, and CLIP.

This approach lets developers choose the right balance between **speed, accuracy, and modality support** depending on their workload.

## 4. Natural Querying

When querying, developers don’t need to provide vectors—they provide natural input. Ahnlich AI applies the configured query model, generates embeddings, and communicates with Ahnlich DB.

### Example – Querying with Text:

```
GETSIMN "climate change effects on agriculture" IN news_store
```

Ahnlich AI generates embeddings for the query and performs a similarity search against all stored article embeddings.
This means queries like *“find me similar jazz songs”* or *“show me products like this image”* become possible without manual preprocessing.
