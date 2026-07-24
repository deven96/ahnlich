---
title: Schemas
sidebar_position: 40
---

# Schemas

Schemas provide logical separation for stores in Ahnlich DB and Ahnlich AI. A schema is a namespace: the same store name can exist in different schemas, and store names only need to be unique within their schema.

The default schema is `public`. When a command or generated client request leaves `schema` unset, Ahnlich resolves that request against `public`. An omitted schema does not search or list stores across every schema.

The `public` schema always exists and cannot be dropped.

## Supported Commands

Schema support is available on these DB and AI store management commands:

| Command | Schema behavior |
| --- | --- |
| `CREATESTORE ... SCHEMA <schema>` | Creates the store in the named schema. Omitting `SCHEMA` creates it in `public`. |
| `GETSTORE <store> SCHEMA <schema>` | Reads a store from the named schema. Omitting `SCHEMA` reads from `public`. |
| `DROPSTORE <store> IF EXISTS SCHEMA <schema>` | Drops a store from the named schema. Omitting `SCHEMA` drops from `public`. |
| `LISTSTORES SCHEMA <schema>` | Lists stores in the named schema. Omitting `SCHEMA` lists only `public`. |
| `DROPSCHEMA <schema>` | Drops a non-public schema and all stores in it. |

## DB Examples

```text
CREATESTORE articles DIMENSION 384 PREDICATES (author, category) SCHEMA analytics
LISTSTORES SCHEMA analytics
GETSTORE articles SCHEMA analytics
DROPSTORE articles IF EXISTS SCHEMA analytics
DROPSCHEMA analytics
```

## AI Examples

```text
CREATESTORE images QUERYMODEL clip-vit-b32-image INDEXMODEL clip-vit-b32-image STOREORIGINAL SCHEMA media
LISTSTORES SCHEMA media
GETSTORE images SCHEMA media
DROPSTORE images IF EXISTS SCHEMA media
DROPSCHEMA media
```

## Generated Clients

The generated Go, Python, and Node.js clients expose `schema` on `CreateStore`, `GetStore`, `DropStore`, and `ListStores`, plus a `DropSchema` request/RPC for DB and AI.

### Go

```go
schema := "analytics"

_, err := client.CreateStore(ctx, &dbquery.CreateStore{
    Store: "articles",
    Dimension: 384,
    Schema: &schema,
})

stores, err := client.ListStores(ctx, &dbquery.ListStores{Schema: &schema})

_, err = client.DropSchema(ctx, &dbquery.DropSchema{Schema: schema})
```

### Python

```py
schema = "analytics"

await client.create_store(
    db_query.CreateStore(store="articles", dimension=384, schema=schema)
)

stores = await client.list_stores(db_query.ListStores(schema=schema))

await client.drop_schema(db_query.DropSchema(schema=schema))
```

### Node.js

```ts
const schema = "analytics";

await client.createStore(
  new CreateStore({ store: "articles", dimension: 384, schema }),
);

const stores = await client.listStores(new ListStores({ schema }));

await client.dropSchema(new DropSchema({ schema }));
```

## AI and DB Coordination

Ahnlich AI stores are backed by DB stores. When AI creates or drops a store with a schema, the backing DB request uses the same schema. Dropping a schema through AI removes the backing DB schema first, then removes AI stores in that schema.

`PurgeStores` is intentionally broad and removes AI stores across schemas, including their backing DB stores.
