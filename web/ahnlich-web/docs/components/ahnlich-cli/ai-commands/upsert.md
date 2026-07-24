---
title: Upsert
sidebar_label: Upsert
---

# Upsert

Update a single entry matching a predicate. Always merges metadata (preserves AI-generated fields).

Run it after connecting the CLI with `--agent ai`.

```bash
UPSERT VALUE {tags: cat,outdoors} IN my_store WHERE (filename = photo.jpg)

# re-embed with new input
UPSERT KEY [updated image bytes] IN my_store WHERE (id = 42) PREPROCESSACTION modelpreprocessing
```
