---
title: End-to-end flow
sidebar_label: End-to-end flow
---

# End-to-end flow

Putting the pieces together — create a store, insert data, build indexes, and
query.

## 1. Create a store

```bash
CREATE STORE articles
```

## 2. Insert data

```bash
SET doc1 [0.12,0.33,0.44] WITH {"topic":"ai","visibility":"public"}
SET doc2 [0.50,0.61,0.11] WITH {"topic":"finance","visibility":"public"}
```

## 3. Build indexes

```bash
CREATE PREDICATE INDEX ON articles(topic)
```

## 4. Run queries

```bash
GETSIMN 3 WITH [0.20,0.10,0.70] USING cosinesimilarity IN articles WHERE (topic="ai")
```

## Related

- [Command reference](/docs/components/ahnlich-db/reference)
- [Similarity algorithms](/docs/components/ahnlich-db/advanced/similarity-algorithms)
- [Quickstart: Ahnlich DB](/docs/getting-started/quickstart-db)
