# Multi-Schema

Stores in ahnlich can be organized into named schemas, providing lightweight namespacing. Every store belongs to exactly one schema.

## Default Schema

When a store is created without specifying a schema, it is placed into the `"public"` schema. The `"public"` schema is always available and cannot be dropped.

## Operations

### CreateStore

Takes an optional `schema` field. If omitted (or set to `None`), the store is created in the `"public"` schema:

```
CREATESTORE my_store DIMENSION 2 PREDICATES (author, country)
```

To create in a custom schema, use the `SCHEMA` keyword:

```
CREATESTORE my_store DIMENSION 2 PREDICATES (author, country) SCHEMA my_schema
```

### GetStore

Takes an optional `schema` field:

- If `schema` is provided, the store is looked up within that schema only.
- If `schema` is `None`, the server searches all schemas in an arbitrary order and returns the first match. An exact-match lookup (where the store is in `"public"` and `schema` is `None`) takes priority.

### DropStore

Takes an optional `schema` field. If `None`, the search logic follows the same rules as GetStore.

### ListStores

Takes an optional `schema` field:

- If `schema` is provided, only stores within that schema are returned.
- If `schema` is `None`, all stores across all schemas are returned.

### DropSchema *(new RPC)*

Drops all stores within a schema and removes the schema itself.

- Dropping `"public"` returns an `InvalidArgument` error.
- Dropping a non-existent schema returns an error.
- Dropping a schema cascades: all stores in that schema are removed from both the AI proxy and the underlying DB.

## Protobuf

The `schema` field and `DropSchema` RPC were added to all relevant messages in the protobuf definitions under `protos/`:

**`protos/db/query.proto`** and **`protos/ai/query.proto`**:
- `CreateStore`, `GetStore`, `DropStore`, `ListStores` — each got an optional `string schema = ...` field (the field number varies by message).
- `DropSchema` — new message with a `string schema` field.

**`protos/services/db_service.proto`** and **`protos/services/ai_service.proto`**:
- New `rpc DropSchema(DropSchema) returns (DropSchemaResponse);` on both services.

## Architecture

- The DB engine stores stores as `HashMap<Schema, HashMap<StoreName, Store>>` — the schema is the outer key.
- The AI proxy mirrors this structure: `ConcurrentHashMap<Schema, ConcurrentHashMap<StoreName, AIStore>>`.
- When `DropSchema` is called on the AI proxy, it cascades to the DB by both dropping the schema remotely and clearing all stores in that schema locally.
- `purge_stores` (DestroyDatabase) clears all schemas and all stores across all schemas.

## CLI

The CLI DSL supports the `SCHEMA` keyword in `CREATESTORE`, `GETSTORE`, `DROPSTORE`, and `LISTSTORES`. It also supports `DROPSCHEMA` as a new top-level command:

```
DROPSCHEMA my_schema
```

## SDK

All SDKs (Go, Python, Node.js) have regenerated protobuf stubs that include the `schema` field and `DropSchema` RPC. The hand-written client wrappers have not been modified — usage requires constructing the protobuf messages directly or using the generated service stubs.
