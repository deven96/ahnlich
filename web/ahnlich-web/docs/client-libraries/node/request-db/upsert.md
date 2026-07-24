---
title: Upsert
---

# Upsert

## Schema

This request accepts an optional `schema` field. When it is omitted, the server uses the `public` schema. Set `schema` to target a store in another schema.

The Upsert request updates a single entry matching a predicate condition. It errors if the predicate matches 0 or multiple entries.

* **Input**: Store name, predicate condition, optional new key/value, merge flag.

* **Behavior**: Updates exactly one matching entry. Errors on 0 or multiple matches.

* **Response**: Upsert counts (inserted and updated).

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { Upsert } from "ahnlich-client-node/grpc/db/query_pb";
import { StoreValue } from "ahnlich-client-node/grpc/keyval_pb";
import { MetadataValue } from "ahnlich-client-node/grpc/metadata_pb";
import { PredicateCondition, Predicate, Equals } from "ahnlich-client-node/grpc/predicates_pb";

async function upsertEntry() {
  const client = createDbClient("127.0.0.1:1369");

  const condition = new PredicateCondition({
    kind: {
      case: "value",
      value: new Predicate({
        kind: {
          case: "equals",
          value: new Equals({
            key: "id",
            value: new MetadataValue({ value: { case: "rawString", value: "123" } }),
          }),
        },
      }),
    },
  });

  const newValue = new StoreValue({
    value: {
      status: new MetadataValue({ value: { case: "rawString", value: "published" } }),
    },
  });

  const response = await client.upsert(
    new Upsert({
      store: "my_store",
      schema: "analytics",
      condition,
      newValue,
      mergeMetadata: true,
    })
  );

  console.log("Updated:", response.upsert?.updated);
}

upsertEntry();
```
</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `store` | `string` | Yes | The name of the store |
| `condition` | `PredicateCondition` | Yes | Must match exactly one entry |
| `newKey` | `StoreKey` | No | New vector to replace existing key |
| `newValue` | `StoreValue` | No | Metadata to update |
| `mergeMetadata` | `boolean` | No | If true, merges metadata. If false, replaces (default: false) |
| `schema` | `string` | No | Schema namespace (defaults to "public") |

## Behavior

- Predicate must match exactly one entry
- `mergeMetadata: true` merges new metadata into existing fields
- `mergeMetadata: false` replaces metadata entirely
- Errors if 0 or multiple entries match
- Updated entries are immediately available for queries
