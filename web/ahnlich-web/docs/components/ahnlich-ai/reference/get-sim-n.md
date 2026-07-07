---
title: GETSIMN
sidebar_label: GETSIMN
---

# GETSIMN

Retrieve the top **N** entries most similar to a **raw input** (embedded on the fly). Optionally filter with a predicate.

## Syntax

```bash
GETSIMN <n> WITH [<raw input>] USING <cosinesimilarity|euclideandistance|hnsw> IN <store> WHERE (<predicate>)
GETSIMN <n> WITH [<raw input>] USING <algorithm> IN <store> SCHEMA media WHERE (<predicate>)
```

## Example

```bash
GETSIMN 3 WITH [renewable energy storage] USING cosinesimilarity IN article_store WHERE (category != sports)
```

---

See [SDK examples](/docs/stores/get-simn) (choose the **AI proxy** tab) for Python, Node, Go, and Rust.
