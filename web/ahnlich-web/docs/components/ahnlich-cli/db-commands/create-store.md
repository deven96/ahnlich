---
title: Create a store
sidebar_label: Create a store
---

# Create a store

Create a DB store with a fixed dimension and optional metadata predicates.

Run it after connecting the CLI with `--agent db`.

```bash
CREATESTORE my_store DIMENSION 128 PREDICATES (author, category)
CREATESTORE my_store DIMENSION 128 PREDICATES (author, category) SCHEMA analytics
```
