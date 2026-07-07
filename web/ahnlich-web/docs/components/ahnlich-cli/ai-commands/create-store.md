---
title: Create a store
sidebar_label: Create a store
---

# Create a store

Create an AI store with query/index models, predicates, an optional index, and `STOREORIGINAL`.

Run it after connecting the CLI with `--agent ai`.

```bash
CREATESTORE my_store QUERYMODEL resnet-50 INDEXMODEL resnet-50 PREDICATES (author, category) NONLINEARALGORITHMINDEX (hnsw) STOREORIGINAL
```
