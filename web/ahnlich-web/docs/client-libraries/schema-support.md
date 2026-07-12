---
title: Schema Support
---

# Schema Support

Generated clients expose schema support directly from the protobuf API.

For DB and AI, every request that targets a store by name accepts an optional `schema` field. This includes store lifecycle commands, data commands, retrieval commands such as `GetSimN`, and index commands.

If `schema` is omitted, the server uses `public`. `ListStores` without a schema returns only stores in `public`; it does not list stores from every schema.

Both DB and AI clients also include `DropSchema`, which removes a non-public schema and all stores inside it. The `public` schema cannot be dropped.

## Rust

```rust
let schema = "analytics".to_string();

db_client
    .create_store(
        CreateStore {
            store: "articles".to_string(),
            dimension: 384,
            schema: Some(schema.clone()),
            ..Default::default()
        },
        None,
    )
    .await?;

db_client
    .set(
        Set {
            store: "articles".to_string(),
            inputs: entries,
            schema: Some(schema.clone()),
            ..Default::default()
        },
        None,
    )
    .await?;

let stores = db_client
    .list_stores_with_schema(Some(schema.clone()), None)
    .await?;

db_client.drop_schema(schema.clone(), None).await?;
```

On the AI client, `DropSchema` takes the request struct and clears the schema on the
proxy and the DB in one call. Dropping it on the DB client alone leaves the proxy's
store registry pointing at stores that are gone.

```rust
ai_client
    .drop_schema(DropSchema { schema }, None)
    .await?;
```

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
