---
title: GETSTORE
sidebar_label: GETSTORE
---

# GETSTORE

Detailed info about a store — its query model, index model, embedding size, and (when connected to a DB instance) the backing `db_info`. Errors if the store doesn't exist.

## Syntax

```bash
GETSTORE <store_name>
GETSTORE <store_name> SCHEMA media
```

## Example

```bash
GETSTORE my_store
```

---

See [SDK examples](/docs/stores/get-store) (choose the **AI proxy** tab) for Python, Node, Go, and Rust.
