---
title: Insert data
sidebar_label: Insert data
---

# Insert data

Insert one or more records, each a key vector plus metadata.

Run it after connecting the CLI with `--agent db`.

```bash
SET ((key1, {author: Alice, category: ml}),(key2, {author: Bob, category: dev})) IN my_store
```
