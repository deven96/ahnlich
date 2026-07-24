---
title: DELETE PREDICATE
sidebar_label: DELETE PREDICATE
---

# DELETE PREDICATE

Delete **all** vectors whose metadata matches a predicate.

## Syntax

```bash
DELETE PREDICATE <predicate> IN <store_name>
DELETE PREDICATE <predicate> IN <store_name> SCHEMA analytics
```

## Example

```bash
DELETE PREDICATE (category = "archive") IN my_store
```

---

See [SDK examples](/docs/stores/delete-predicate) for Python, Node, Go, and Rust.
