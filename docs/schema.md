# Multi-Schema Support

Ahnlich stores can be grouped into named schemas. A schema is a lightweight namespace: store names only need to be unique within the same schema, and each store belongs to exactly one schema.

## Default Schema

`public` is the default schema. When a request omits the optional `schema` field, the server resolves the operation against `public`.

The `public` schema is always available and cannot be dropped.

## Schema Names

Schema names use the same CLI-friendly identifier shape as store names: letters, numbers, `_`, and `-`.

Examples:

```text
analytics
tenant_1
customer-eu
```

## Operations

All commands that operate on a store by name accept an optional `schema` field. If `schema` is omitted, the operation targets `public`.

### Store Lifecycle

`CreateStore`, `GetStore`, and `DropStore` accept an optional `schema` field.

DB DSL:

```text
CREATESTORE articles DIMENSION 384 PREDICATES (author, category)
CREATESTORE articles DIMENSION 384 PREDICATES (author, category) SCHEMA analytics
```

AI DSL:

```text
CREATESTORE images QUERYMODEL clip-vit-b32-image INDEXMODEL clip-vit-b32-image STOREORIGINAL
CREATESTORE images QUERYMODEL clip-vit-b32-image INDEXMODEL clip-vit-b32-image STOREORIGINAL SCHEMA media
```

### GetStore

`GetStore` accepts an optional `schema` field.

If `schema` is omitted, the lookup is performed in `public`.

```text
GETSTORE articles
GETSTORE articles SCHEMA analytics
```

### DropStore

`DropStore` accepts an optional `schema` field.

If `schema` is omitted, the store is dropped from `public`.

```text
DROPSTORE articles
DROPSTORE articles IF EXISTS SCHEMA analytics
```

### ListStores

`ListStores` accepts an optional `schema` field.

If `schema` is omitted, only stores in `public` are returned. Omitting the schema does not list every store in every schema.

```text
LISTSTORES
LISTSTORES SCHEMA analytics
```

### DropSchema

`DropSchema` removes a non-public schema and all stores inside it.

```text
DROPSCHEMA analytics
```

Rules:

- Dropping `public` returns an `InvalidArgument` error.
- Dropping a schema that does not exist returns an error.
- Dropping a schema cascades through the service that receives the request. For AI, the proxy now drops the schema in the backing DB first, then removes the local AI stores for that schema.

### Store Data And Index Commands

The same optional schema field is available on store data, retrieval, and index commands:

- DB: `Set`, `GetKey`, `GetPred`, `GetSimN`, `DelKey`, `DelPred`, `CreatePredIndex`, `DropPredIndex`, `CreateNonLinearAlgorithmIndex`, and `DropNonLinearAlgorithmIndex`.
- AI: `Set`, `GetKey`, `GetPred`, `GetSimN`, `DelKey`, `DelPred`, `CreatePredIndex`, `DropPredIndex`, `CreateNonLinearAlgorithmIndex`, and `DropNonLinearAlgorithmIndex`.

DB DSL examples:

```text
SET (([0.1, 0.2], {category: news})) IN articles SCHEMA analytics
GETKEY ([0.1, 0.2]) IN articles SCHEMA analytics
GETPRED (category = news) IN articles SCHEMA analytics
GETSIMN 3 WITH [0.1, 0.2] USING cosinesimilarity IN articles SCHEMA analytics WHERE (category = news)
DELKEY ([0.1, 0.2]) IN articles SCHEMA analytics
CREATEPREDINDEX (category) IN articles SCHEMA analytics
DROPPREDINDEX (category) IN articles SCHEMA analytics
CREATENONLINEARALGORITHMINDEX (hnsw) IN articles SCHEMA analytics
DROPNONLINEARALGORITHMINDEX (hnsw) IN articles SCHEMA analytics
```

AI DSL examples:

```text
SET (([document text], {category: news})) IN articles SCHEMA analytics PREPROCESSACTION nopreprocessing
GETPRED (category = news) IN articles SCHEMA analytics
GETSIMN 3 WITH [document text] USING cosinesimilarity IN articles SCHEMA analytics WHERE (category = news)
DELKEY ([document text]) IN articles SCHEMA analytics
CREATEPREDINDEX (category) IN articles SCHEMA analytics
DROPPREDINDEX (category) IN articles SCHEMA analytics
CREATENONLINEARALGORITHMINDEX (hnsw) IN articles SCHEMA analytics
DROPNONLINEARALGORITHMINDEX (hnsw) IN articles SCHEMA analytics
```

## Protobuf

The schema feature is exposed in the protobuf definitions under `protos/`.

`protos/db/query.proto` and `protos/ai/query.proto`:

- All store-name commands include an optional `string schema` field.
- `ListStores` includes an optional `string schema` field and defaults to `public` when unset.
- `DropSchema` includes a required `string schema` field.

`protos/db/pipeline.proto` and `protos/ai/pipeline.proto`:

- Pipelines include `DropSchema` query variants.

`protos/services/db_service.proto` and `protos/services/ai_service.proto`:

- Both services expose `rpc DropSchema(...)`.

## CLI DSL

The CLI DSL supports schema parsing for both DB and AI commands:

```text
CREATESTORE ... SCHEMA <schema_name>
GETSTORE <store_name> SCHEMA <schema_name>
DROPSTORE <store_name> IF EXISTS SCHEMA <schema_name>
LISTSTORES SCHEMA <schema_name>
GETSIMN ... IN <store_name> SCHEMA <schema_name> WHERE ...
GETPRED ... IN <store_name> SCHEMA <schema_name>
SET ... IN <store_name> SCHEMA <schema_name>
DROPSCHEMA <schema_name>
```

The `SCHEMA` keyword is optional for store-name commands. If it is omitted, the command targets `public`.

`DROPSCHEMA` always requires a schema name.

## Architecture

The DB engine stores data as a schema map containing per-schema store maps:

```text
HashMap<Schema, HashMap<StoreName, Store>>
```

The AI proxy mirrors the same shape for AI stores:

```text
ConcurrentHashMap<Schema, ConcurrentHashMap<StoreName, AIStore>>
```

Schema-aware operations resolve the schema before looking up or mutating the store. For public-default operations, that resolved schema is `public`.

`PurgeStores` still clears all schemas and all stores.

## SDKs

The generated Go, Python, Node.js, and Rust protobuf stubs include the schema fields and `DropSchema` RPC.

Generated client calls should set `schema` on the request when targeting a non-public schema. Leaving it unset preserves the public default.
