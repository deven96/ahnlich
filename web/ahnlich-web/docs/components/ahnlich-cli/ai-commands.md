---
title: AI Commands
sidebar_label: AI commands
---

# Ahnlich CLI — AI commands

Run queries against **AI stores** from the CLI. Unlike DB stores, you send raw
**text or images** and the proxy embeds them for you. Connect first with
`--agent ai`:

```bash
ahnlich-cli ahnlich --agent ai --host 127.0.0.1 --port 1370
```

## Example workflow

1. **Create a store** with query/index models and predicates.
2. **Insert** raw text or images — embeddings are generated automatically.
3. **Query** by similarity or predicate.
4. **Manage indexes** for faster searches.
5. **Drop** stores or keys when they're no longer needed.

## Commands

**Server & stores** — [Ping](./ai-commands/ping) · [Server info](./ai-commands/infoserver) · [List stores](./ai-commands/list-stores) · [Create a store](./ai-commands/create-store) · [Drop a store](./ai-commands/drop-store) · [Drop a schema](./ai-commands/drop-schema)

**Data** — [Insert data](./ai-commands/set) · [Upsert](./ai-commands/upsert) · [Query by similarity](./ai-commands/get-sim-n) · [Query by predicate](./ai-commands/get-pred) · [Delete a key](./ai-commands/delete-key)

**Indexes** — [Create predicate index](./ai-commands/create-predicate-index) · [Drop predicate index](./ai-commands/drop-predicate-index) · [Create non-linear index](./ai-commands/create-non-linear-index) · [Drop non-linear index](./ai-commands/drop-non-linear-index)
