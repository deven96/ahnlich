---
title: GETSIMN
sidebar_label: GETSIMN
---

# GETSIMN

Find the **N** most similar vectors to an input vector, ranked by a similarity metric. Optionally restrict candidates with a predicate.

## Syntax

```bash
GETSIMN <n> WITH [<float>, ...] USING <cosinesimilarity|euclideandistance|hnsw> IN <store_name> WHERE (<predicate>)
```

## Parameters

- `USING` picks the algorithm — see [Vector index](/docs/concepts/vector-index) and [Similarity metrics](/docs/concepts/similarity-metrics).

## Example

```bash
GETSIMN 3 WITH [0.25, 0.88] USING cosinesimilarity IN my_store WHERE (category != "draft")
```

---

See [SDK examples](/docs/stores/get-simn) for Python, Node, Go, and Rust.
