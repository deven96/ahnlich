---
title: CREATESTORE
sidebar_label: CREATESTORE
---

# CREATESTORE

Create a new store with a fixed **dimension** — the length every vector in it must have.

## Syntax

```bash
CREATESTORE <store_name> DIMENSION <n>
CREATESTORE <store_name> DIMENSION <n> SCHEMA analytics
```

## Example

```bash
CREATESTORE my_store DIMENSION 128
```

---

See [SDK examples](/docs/stores/create-store) for Python, Node, Go, and Rust.
