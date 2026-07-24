---
title: Insert data
sidebar_label: Insert data
---

# Insert data

Insert text, images, or audio with metadata — the proxy embeds them. `PREPROCESSACTION` controls preprocessing.

Run it after connecting the CLI with `--agent ai`.

```bash
SET (([This is the life of Alice], {author: Alice, category: ml}),([This is the life of Bob], {author: Bob, category: dev})) IN my_store PREPROCESSACTION nopreprocessing
```
