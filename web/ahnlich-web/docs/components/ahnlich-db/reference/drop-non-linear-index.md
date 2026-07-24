---
title: DROP NON LINEAR ALGORITHM INDEX
sidebar_label: DROP NON LINEAR ALGORITHM INDEX
---

# DROP NON LINEAR ALGORITHM INDEX

Remove a non-linear index. Similarity search still works, falling back to a linear scan.

## Syntax

```bash
DROP NON LINEAR ALGORITHM INDEX <algorithm> IN <store_name>
DROP NON LINEAR ALGORITHM INDEX <algorithm> IN <store_name> SCHEMA analytics
```

## Example

```bash
DROP NON LINEAR ALGORITHM INDEX hnsw IN my_store
```

---

See [SDK examples](/docs/stores/create-non-linear-algx) for Python, Node, Go, and Rust.
