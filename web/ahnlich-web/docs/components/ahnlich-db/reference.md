---
title: Command reference
sidebar_label: Overview
sidebar_position: 30
---

# DB command reference

Every command Ahnlich DB understands, one page each, grouped by what it does. These
are the **text-query / CLI** forms; for full, runnable **SDK** examples in Python,
Node, Go, and Rust, each page links to its matching [Stores](/docs/stores)
operation.

## Server & system

- [PING](./reference/ping) · [INFOSERVER](./reference/infoserver) · [LIST CONNECTED CLIENTS](./reference/list-connected-clients)

## Store management

- [LISTSTORES](./reference/list-stores) · [GETSTORE](./reference/get-store) · [CREATESTORE](./reference/create-store) · [DROPSTORE](./reference/drop-store) · [DROPSCHEMA](./reference/drop-schema)

## Data operations

- [SET](./reference/set) · [UPSERT](./reference/upsert) · [DELETE KEY](./reference/delete-key) · [DELETE PREDICATE](./reference/delete-predicate)

## Query & retrieval

- [GETSIMN](./reference/get-sim-n) · [GETKEY](./reference/get-key) · [GET BY PREDICATE](./reference/get-by-predicate)

## Index management

- [CREATE PREDICATE INDEX](./reference/create-predicate-index) · [DROP PREDICATE INDEX](./reference/drop-predicate-index) · [CREATE NON LINEAR ALGORITHM INDEX](./reference/create-non-linear-index) · [DROP NON LINEAR ALGORITHM INDEX](./reference/drop-non-linear-index)

## Reference

- [Command → SDK map](./reference/sdk-mapping) — every command next to its Rust / Python / Go call.

---

**How to read this reference**

- `UPPERCASE` is a keyword; `<angle brackets>` are values you supply.
- `[SCHEMA <schema>]` is optional — omit it to target the `public`
  [schema](/docs/concepts/schemas).
- Every command works over the text query interface, the
  [CLI](/docs/components/ahnlich-cli), or a client SDK.
