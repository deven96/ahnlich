---
title: Drop a store
sidebar_label: Drop a store
---

# Drop a store

Delete a store and its contents. `IF EXISTS` avoids an error if it's gone.

Run it after connecting the CLI with `--agent db`.

```bash
DROPSTORE my_store IF EXISTS
DROPSTORE my_store IF EXISTS SCHEMA analytics
```
