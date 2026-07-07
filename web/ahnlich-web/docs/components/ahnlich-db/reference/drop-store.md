---
title: DROPSTORE
sidebar_label: DROPSTORE
---

# DROPSTORE

Permanently delete a store and all its contents. Add `IF EXISTS` to avoid an error when it's already gone.

## Syntax

```bash
DROPSTORE <store_name>
DROPSTORE <store_name> IF EXISTS SCHEMA analytics
```

## Example

```bash
DROPSTORE my_store IF EXISTS
```

---

See [SDK examples](/docs/stores/drop-store) for Python, Node, Go, and Rust.
