---
title: Command reference
sidebar_label: Overview
sidebar_position: 20
---

# AI command reference

Ahnlich AI is the proxy that turns **raw input** — text, images, or audio — into
embeddings and manages them in vector stores backed by
[Ahnlich DB](/docs/components/ahnlich-db). Every command it understands is listed
below, one page each. The bundled model is `all-minilm-l6-v2` (text).

For full, runnable **SDK** examples, each page links to its matching
[Stores](/docs/stores) operation — pick the **AI proxy** tab.

## Server & system

- [PING](./reference/ping) · [INFOSERVER](./reference/infoserver)

## Store management

- [LISTSTORES](./reference/list-stores) · [GETSTORE](./reference/get-store) · [CREATESTORE](./reference/create-store) · [DROPSTORE](./reference/drop-store) · [DROPSCHEMA](./reference/drop-schema)

## Data operations

- [SET](./reference/set) · [UPSERT](./reference/upsert) · [DELETEKEY](./reference/delete-key)

## Query & retrieval

- [GETSIMN](./reference/get-sim-n) · [GETPRED](./reference/get-by-predicate) · [GETKEY](./reference/get-key)

## Index management

- [CREATEPREDICATEINDEX](./reference/create-predicate-index) · [DROPPREDICATEINDEX](./reference/drop-predicate-index) · [CREATENONLINEARALGORITHMINDEX](./reference/create-non-linear-index) · [DROPNONLINEARALGORITHMINDEX](./reference/drop-non-linear-index)

## Reference

- [Command → SDK map](./reference/sdk-mapping)

---

**How Ahnlich AI differs from Ahnlich DB:** you send **text, images, or audio**, not vectors —
the proxy embeds them with the store's models. The commands otherwise mirror
[Ahnlich DB](/docs/components/ahnlich-db/reference).
