---
title: Command deep dive
sidebar_label: Command deep dive
---

# Command deep dive

A power-user's walkthrough of how commands behave in practice. For the full
per-command reference, see
[Command reference](/docs/components/ahnlich-db/reference).

## Server management

**PING** — test whether the DB is alive. Essential for monitoring.

```text
> PING
< PONG
```

**INFO SERVER** — server metadata: version, uptime, active stores, tracing status.

```text
> INFO SERVER
< {"version":"0.0.2","uptime":"3h45m","stores":["docs","images"]}
```

**LIST CONNECTED CLIENTS** — all clients with IP and connection status; useful for
debugging distributed workloads.

## Store lifecycle

**LIST STORES** — the stores available, with name, entry count, size in bytes, and
any non-linear index configuration.

**CREATE STORE `<name>`** — a new container for vectors + metadata. Optionally
accepts non-linear index configs (HNSW parameters).

```text
> CREATE STORE articles
< OK
```

**DROP STORE `<name>`** — deletes permanently. Data can't be recovered unless
[persistence](/docs/components/persistence-in-ahnlich) is enabled.

## Vector operations

**SET** — insert or overwrite a vector + metadata.

```text
> SET doc1 [0.12, 0.33, 0.44] WITH {"topic":"ai","visibility":"public"}
< OK
```

**GET KEY** — retrieve a vector and metadata by key.

```text
> GET KEY doc1
< {"vector":[0.12,0.33,0.44],"metadata":{"topic":"ai","visibility":"public"}}
```

**DELETE KEY** — remove a vector completely.

## Querying & filtering

**GET SIM N** — the core similarity query. Finds the N closest vectors, supports
linear (`cosine`, `euclidean`) and non-linear (`hnsw`), and can apply
metadata filters.

```text
> GETSIMN 3 WITH [0.2,0.1,0.7] USING cosinesimilarity IN articles WHERE (visibility = "public")
< [{"key":"doc5","score":0.92},{"key":"doc3","score":0.89},{"key":"doc7","score":0.87}]
```

**GET BY PREDICATE** — filter on metadata without similarity search.

```text
> GET BY PREDICATE topic = "ai" IN articles
```

**DELETE PREDICATE** — bulk delete by metadata.

```text
> DELETE PREDICATE visibility = "hidden" IN articles
```

## Indexes

**CREATE / DROP PREDICATE INDEX** — speed up (or clean up) metadata filtering.

```text
> CREATE PREDICATE INDEX ON articles(topic)
```

**CREATE / DROP NON LINEAR ALGORITHM INDEX** — build or remove an HNSW
index for nearest-neighbour queries.

```text
> CREATE NON LINEAR ALGORITHM INDEX hnsw ON semantic_store
```

## Related

- [Command reference](/docs/components/ahnlich-db/reference)
- [End-to-end flow](/docs/components/ahnlich-db/advanced/end-to-end-flow)
