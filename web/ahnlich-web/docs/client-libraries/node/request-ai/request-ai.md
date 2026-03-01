---
title: Request AI
sidebar_position: 1
---

# Request AI

This section covers all available AI operations for the Node.js SDK when interacting with Ahnlich AI.

Ahnlich AI is the semantic layer that provides:

* Automatic embedding generation using AI models
* Text and image similarity search
* Semantic understanding without manual vector management

## Available Operations

### Server Operations
- [Ping](/docs/client-libraries/node/request-ai/ping) - Health check
- [Info Server](/docs/client-libraries/node/request-ai/info-server) - Get server information

### Store Operations
- [List Stores](/docs/client-libraries/node/request-ai/list-stores) - List all AI stores
- [Get Store](/docs/client-libraries/node/request-ai/get-store) - Get details of a specific AI store
- [Create Store](/docs/client-libraries/node/request-ai/create-store) - Create a new AI store
- [Drop Store](/docs/client-libraries/node/request-ai/drop-store) - Delete an AI store

### Data Operations
- [Set](/docs/client-libraries/node/request-ai/set) - Insert entries (auto-generates embeddings)
- [GetSimN](/docs/client-libraries/node/request-ai/get-simn) - Semantic similarity search
- [Get By Predicate](/docs/client-libraries/node/request-ai/get-by-predicate) - Filter entries by metadata
- [Delete Key](/docs/client-libraries/node/request-ai/delete-key) - Delete entries by input

### Index Operations
- [Create Predicate Index](/docs/client-libraries/node/request-ai/create-predicate-index) - Create metadata index
- [Drop Predicate Index](/docs/client-libraries/node/request-ai/drop-predicate-index) - Remove metadata index
- [Create Non Linear Algorithm Index](/docs/client-libraries/node/request-ai/create-non-linear-algx) - Create KDTree/HNSW index
- [Drop Non Linear Algorithm Index](/docs/client-libraries/node/request-ai/drop-non-linear-algx) - Remove KDTree/HNSW index

## Key Differences from DB

- **Automatic embeddings**: You provide text/images, AI generates vectors
- **Semantic search**: Search by meaning, not exact vectors
- **Model selection**: Choose from supported AI models (MiniLM, ResNet, CLIP, etc.)
