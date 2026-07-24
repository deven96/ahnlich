---
title: GETPRED
sidebar_label: GETPRED
---

# GETPRED

Retrieve every entry that satisfies a metadata condition.

## Syntax

```bash
GETPRED (<predicate>) IN <store>
GETPRED (<predicate>) IN <store> SCHEMA media
```

## Example

```bash
GETPRED (category = "news") IN article_store
```

---

See [SDK examples](/docs/stores/get-by-predicate) (choose the **AI proxy** tab) for Python, Node, Go, and Rust.
