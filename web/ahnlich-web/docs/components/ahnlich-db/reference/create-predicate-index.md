---
title: CREATE PREDICATE INDEX
sidebar_label: CREATE PREDICATE INDEX
---

# CREATE PREDICATE INDEX

Index a metadata field so predicate filters run as a direct lookup instead of a full scan.

## Syntax

```bash
CREATE PREDICATE INDEX <field> IN <store_name>
CREATE PREDICATE INDEX <field> IN <store_name> SCHEMA analytics
```

## Example

```bash
CREATE PREDICATE INDEX category IN my_store
```

---

See [SDK examples](/docs/stores/create-predicate-index) for Python, Node, Go, and Rust.
