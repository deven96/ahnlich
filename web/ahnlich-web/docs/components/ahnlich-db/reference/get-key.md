---
title: GETKEY
sidebar_label: GETKEY
---

# GETKEY

Retrieve a vector and its metadata by exact key — no similarity ranking.

## Syntax

```bash
GETKEY ([<float>, <float>]) IN <store_name>
GETKEY ([<float>, <float>]) IN <store_name> SCHEMA analytics
```

## Example

```bash
GETKEY ([1.0, 2.0]) IN my_store
```

---

See [SDK examples](/docs/stores/get-key) for Python, Node, Go, and Rust.
