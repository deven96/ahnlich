---
title: DROPNONLINEARALGORITHMINDEX
sidebar_label: DROPNONLINEARALGORITHMINDEX
---

# DROPNONLINEARALGORITHMINDEX

Drop a previously created non-linear index.

## Syntax

```bash
DROPNONLINEARALGORITHMINDEX (<algorithm>) IN <store>
DROPNONLINEARALGORITHMINDEX (<algorithm>) IN <store> SCHEMA media
```

## Example

```bash
DROPNONLINEARALGORITHMINDEX (hnsw) IN geo_store
```

---

See [SDK examples](/docs/stores/drop-non-linear-algx) (choose the **AI proxy** tab) for Python, Node, Go, and Rust.
