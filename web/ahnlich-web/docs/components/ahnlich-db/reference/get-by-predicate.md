---
title: GET BY PREDICATE
sidebar_label: GET BY PREDICATE
---

# GET BY PREDICATE

Retrieve every entry whose metadata satisfies a predicate — filtering by attributes, not by vector.

## Syntax

```bash
GET BY PREDICATE (<predicate>) IN <store_name>
GET BY PREDICATE (<predicate>) IN <store_name> SCHEMA analytics
```

## Example

```bash
GET BY PREDICATE (lang = "en") IN my_store
```

---

See [SDK examples](/docs/stores/get-by-predicate) for Python, Node, Go, and Rust.
