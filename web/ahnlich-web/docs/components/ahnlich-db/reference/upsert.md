---
title: UPSERT
sidebar_label: UPSERT
---

# UPSERT

Update a **single** entry matched by a predicate — replace its vector, its metadata, or both. Errors if zero or multiple entries match.

## Syntax

```bash
UPSERT [MERGE] [KEY <vector>] [VALUE <metadata>] IN <store> [SCHEMA <schema>] WHERE (<predicate>)
```

## Parameters

- `MERGE` — merge new metadata into existing fields (default: replace).
- `KEY <vector>` — a new vector to replace the matched key.
- `VALUE <metadata>` — metadata to update.
- `WHERE (<predicate>)` — must match exactly one entry.

## Example

```bash
UPSERT VALUE {status: published} IN articles WHERE (id = 123)
UPSERT MERGE VALUE {updated_at: 2026-07-02} IN articles WHERE (id = 123)
UPSERT KEY [0.1, 0.2, 0.3] IN embeddings WHERE (doc_id = abc)
```

---

See [SDK examples](/docs/stores/upsert) for Python, Node, Go, and Rust.
