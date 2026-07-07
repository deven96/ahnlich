---
title: Upsert
sidebar_label: Upsert
---

# Upsert

Update a single entry matching a predicate. Errors if 0 or multiple match.

Run it after connecting the CLI with `--agent db`.

```bash
UPSERT VALUE {status: published} IN my_store WHERE (id = 123)

# merge metadata (preserve unchanged fields)
UPSERT MERGE VALUE {updated_at: 2026-07-02} IN my_store WHERE (id = 123)

# update the vector only
UPSERT KEY [0.1, 0.2, 0.3] IN my_store WHERE (doc_id = abc)
```
