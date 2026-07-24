---
title: DB Commands
sidebar_label: DB commands
---

# Ahnlich CLI — Database commands

Run structured queries against **DB stores** from the CLI: insert, retrieve, and
manage key–value vector data with predicates and indexes. Connect first with
`--agent db`:

```bash
ahnlich-cli ahnlich --agent db --host 127.0.0.1 --port 1369
```

## Example workflow

1. **Create a store** with predicates and optional indexes.
2. **Insert data** into the store.
3. **Query** by key or predicate.
4. **Manage indexes** for faster searches.
5. **Drop** stores or keys when they're no longer needed.

## Commands

**Server & stores** — [Ping](./db-commands/ping) · [Server info](./db-commands/infoserver) · [List stores](./db-commands/list-stores) · [Create a store](./db-commands/create-store) · [Drop a store](./db-commands/drop-store) · [Drop a schema](./db-commands/drop-schema)

**Data** — [Insert data](./db-commands/set) · [Upsert](./db-commands/upsert) · [Get by key](./db-commands/get-key) · [Query by predicate](./db-commands/get-pred) · [Delete a key](./db-commands/delete-key)

**Indexes** — [Create predicate index](./db-commands/create-predicate-index) · [Drop predicate index](./db-commands/drop-predicate-index) · [Create non-linear index](./db-commands/create-non-linear-index) · [Drop non-linear index](./db-commands/drop-non-linear-index)
