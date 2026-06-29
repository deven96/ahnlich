---
title: Schema Support
---

# Schema Support

Generated clients expose schema support directly from the protobuf API.

For DB and AI, every request that targets a store by name accepts an optional `schema` field. This includes store lifecycle commands, data commands, retrieval commands such as `GetSimN`, and index commands.

If `schema` is omitted, the server uses `public`. `ListStores` without a schema returns only stores in `public`; it does not list stores from every schema.

Both DB and AI clients also include `DropSchema`, which removes a non-public schema and all stores inside it. The `public` schema cannot be dropped.

## Go

```go
schema := "analytics"

_, err := client.CreateStore(ctx, &dbquery.CreateStore{
    Store: "articles",
    Dimension: 384,
    Schema: &schema,
})

_, err = client.Set(ctx, &dbquery.Set{
    Store: "articles",
    Inputs: entries,
    Schema: &schema,
})

matches, err := client.GetSimN(ctx, &dbquery.GetSimN{
    Store: "articles",
    SearchInput: queryVector,
    ClosestN: 5,
    Algorithm: algorithm.Algorithm_CosineSimilarity,
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

await client.set(db_query.Set(store="articles", inputs=entries, schema=schema))

matches = await client.get_sim_n(
    db_query.GetSimN(
        store="articles",
        search_input=query_vector,
        closest_n=5,
        algorithm=db_algorithm.Algorithm.CosineSimilarity,
        schema=schema,
    )
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

await client.set(new Set({ store: "articles", inputs: entries, schema }));

const matches = await client.getSimN(
  new GetSimN({
    store: "articles",
    searchInput: queryVector,
    closestN: BigInt(5),
    algorithm: Algorithm.COSINE_SIMILARITY,
    schema,
  }),
);

const stores = await client.listStores(new ListStores({ schema }));

await client.dropSchema(new DropSchema({ schema }));
```
