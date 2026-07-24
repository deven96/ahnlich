---
title: CREATENONLINEARALGORITHMINDEX
sidebar_label: CREATENONLINEARALGORITHMINDEX
---

# CREATENONLINEARALGORITHMINDEX

Build a [vector index](/docs/concepts/vector-index) — `hnsw` — for faster similarity search.

## Syntax

```bash
CREATENONLINEARALGORITHMINDEX (<algorithm>) IN <store>
CREATENONLINEARALGORITHMINDEX (<algorithm>) IN <store> SCHEMA media
```

## Example

```bash
CREATENONLINEARALGORITHMINDEX (hnsw) IN geo_store
CREATENONLINEARALGORITHMINDEX (hnsw) IN geo_store
```

---

See [SDK examples](/docs/stores/create-non-linear-algx) (choose the **AI proxy** tab) for Python, Node, Go, and Rust.
