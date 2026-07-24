---
title: LISTSTORES
sidebar_label: LISTSTORES
---

# LISTSTORES

Show the stores in a schema — name, entry count, size, predicate indices, dimension, and non-linear index config.

## Syntax

```bash
LISTSTORES
LISTSTORES SCHEMA analytics
```

## Parameters

- If `SCHEMA` is omitted, only stores in `public` are returned.

## Example

```bash
LISTSTORES
```

---

See [SDK examples](/docs/stores/list-stores) for Python, Node, Go, and Rust.
