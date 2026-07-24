---
title: Query by similarity
sidebar_label: Query by similarity
---

# Query by similarity

Find the top N entries most similar to a raw input, optionally filtered by predicate.

Run it after connecting the CLI with `--agent ai`.

```bash
GETSIMN 4 WITH [This is the life of Alice] USING cosinesimilarity IN my_store WHERE (category = ml)
```
