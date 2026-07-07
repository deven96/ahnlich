---
title: CREATESTORE
sidebar_label: CREATESTORE
---

# CREATESTORE

Create an AI store with an **index model** (embeds stored data) and a **query model** (embeds searches) — usually the same model.

## Syntax

```bash
CREATESTORE <store> QUERYMODEL <model> INDEXMODEL <model>
CREATESTORE <store> QUERYMODEL <model> INDEXMODEL <model> SCHEMA media
```

## Parameters

- The bundled model is `all-minilm-l6-v2` (text).

## Example

```bash
CREATESTORE my_store QUERYMODEL all-minilm-l6-v2 INDEXMODEL all-minilm-l6-v2
```

---

See [SDK examples](/docs/stores/create-store) (choose the **AI proxy** tab) for Python, Node, Go, and Rust.
