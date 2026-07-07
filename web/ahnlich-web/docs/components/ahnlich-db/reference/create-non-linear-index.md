---
title: CREATE NON LINEAR ALGORITHM INDEX
sidebar_label: CREATE NON LINEAR ALGORITHM INDEX
---

# CREATE NON LINEAR ALGORITHM INDEX

Build a [vector index](/docs/concepts/vector-index) — `hnsw` (approximate, high-dim) — to accelerate similarity search.

## Syntax

```bash
CREATE NON LINEAR ALGORITHM INDEX <algorithm> IN <store_name>
CREATE NON LINEAR ALGORITHM INDEX <algorithm> IN <store_name> SCHEMA analytics
```

## Example

```bash
CREATE NON LINEAR ALGORITHM INDEX hnsw IN my_store
```

---

See [SDK examples](/docs/stores/create-non-linear-algx) for Python, Node, Go, and Rust.
