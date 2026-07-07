---
title: SET
sidebar_label: SET
---

# SET

Insert **raw input** (text, an image, or audio) into a store — Ahnlich AI embeds it for you. Add metadata as key–value pairs.

## Syntax

```bash
SET <key> "<raw input>" WITH {"<meta_key>":"<meta_value>"} IN <store>
SET <key> "<raw input>" WITH { ... } IN <store> SCHEMA media
```

## Example

```bash
SET doc1 "The future of AI in healthcare" WITH {"category":"news"} IN article_store
```

---

See [SDK examples](/docs/stores/set) (choose the **AI proxy** tab) for Python, Node, Go, and Rust.
