---
title: UPSERT
sidebar_label: UPSERT
---

# UPSERT

Update a single entry matched by a predicate. Always **merges** metadata (preserving AI-generated fields). Errors if zero or multiple entries match.

## Syntax

```bash
UPSERT [KEY <raw input>] [VALUE <metadata>] IN <store> WHERE (<predicate>) [PREPROCESSACTION <action>]
```

## Parameters

- `KEY <raw input>` — new text, image, or audio to re-embed.
- `VALUE <metadata>` — metadata to merge.
- `PREPROCESSACTION` — how to prepare input before embedding.
- `WHERE (<predicate>)` — must match exactly one entry.

## Example

```bash
UPSERT VALUE {tags: cat,outdoors} IN images WHERE (filename = photo.jpg)
UPSERT KEY [new text] VALUE {author: Jane} IN docs WHERE (id = 100)
```

---

See [SDK examples](/docs/stores/upsert) (choose the **AI proxy** tab) for Python, Node, Go, and Rust.
