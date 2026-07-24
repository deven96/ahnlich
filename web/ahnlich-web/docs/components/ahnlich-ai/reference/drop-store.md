---
title: DROPSTORE
sidebar_label: DROPSTORE
---

# DROPSTORE

Permanently remove an AI store and its contents. `IF EXISTS` avoids an error if it's already gone.

## Syntax

```bash
DROPSTORE <store>
DROPSTORE <store> IF EXISTS SCHEMA media
```

## Example

```bash
DROPSTORE article_store IF EXISTS
```

---

See [SDK examples](/docs/stores/drop-store) (choose the **AI proxy** tab) for Python, Node, Go, and Rust.
