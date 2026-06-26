---
title: Schema Support
---

# Schema Support

Generated clients expose schema support directly from the protobuf API.

For DB and AI, `CreateStore`, `GetStore`, `DropStore`, and `ListStores` accept an optional `schema` field. If `schema` is omitted, the server uses `public`. `ListStores` without a schema returns only stores in `public`.

Both DB and AI clients also include `DropSchema`, which removes a non-public schema and all stores inside it. The `public` schema cannot be dropped.

## Go

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

## Python

```py
schema = "analytics"

await client.create_store(
    db_query.CreateStore(store="articles", dimension=384, schema=schema)
)

stores = await client.list_stores(db_query.ListStores(schema=schema))

await client.drop_schema(db_query.DropSchema(schema=schema))
```

## Node.js

```ts
const schema = "analytics";

await client.createStore(
  new CreateStore({ store: "articles", dimension: 384, schema }),
);

const stores = await client.listStores(new ListStores({ schema }));

await client.dropSchema(new DropSchema({ schema }));
```
