---
title: SET
sidebar_label: SET
---

# SET

Insert or update one or more vectors with metadata. Re-inserting an existing key overwrites its value.

## Syntax

```bash
SET <key> [<float>, ...] WITH { "<meta_key>": "<meta_value>", ... } IN <store_name>
SET <key> [<float>, ...] WITH { ... } IN <store_name> SCHEMA analytics
```

## Parameters

- The vector length must match the store's dimension.

## Example

```bash
SET doc1 [0.25, 0.88] WITH { "category": "news", "lang": "en" } IN my_store
```

---

See [SDK examples](/docs/stores/set) for Python, Node, Go, and Rust.
