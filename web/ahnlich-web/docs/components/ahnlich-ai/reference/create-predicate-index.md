---
title: CREATEPREDICATEINDEX
sidebar_label: CREATEPREDICATEINDEX
---

# CREATEPREDICATEINDEX

Index a metadata field to speed up predicate queries.

## Syntax

```bash
CREATEPREDICATEINDEX <field> IN <store>
CREATEPREDICATEINDEX <field> IN <store> SCHEMA media
```

## Example

```bash
CREATEPREDICATEINDEX category IN article_store
```

---

See [SDK examples](/docs/stores/create-predicate-index) (choose the **AI proxy** tab) for Python, Node, Go, and Rust.
