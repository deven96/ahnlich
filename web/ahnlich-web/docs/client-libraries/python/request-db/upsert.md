---
title: Upsert
---

# Upsert

## Schema

This request accepts an optional `schema` field. When it is omitted, the server uses the `public` schema. Set `schema` to target a store in another schema.

The Upsert request updates a single entry matching a predicate condition:

* **Input**:

  * `store`: the store name.

  * `condition`: predicate that must match exactly one entry.

  * `new_key` (optional): new vector to replace the existing key.

  * `new_value` (optional): metadata to update.

  * `merge_metadata` (optional): if True, merges new metadata into existing (default: False replaces entirely).

* **Behavior**: Updates the matched entry. Errors if 0 or multiple entries match the predicate.

* **Response**: A Set response with upsert counts (inserted: 0, updated: 1).

<details>
  <summary>Click to expand source code</summary>

```py
import asyncio
from grpclib.client import Channel
from ahnlich_client_py.grpc import keyval, metadata, predicates
from ahnlich_client_py.grpc.services.db_service import DbServiceStub
from ahnlich_client_py.grpc.db import query as db_query


async def upsert():
  async with Channel(host="127.0.0.1", port=1369) as channel:
    client = DbServiceStub(channel)

    condition = predicates.PredicateCondition(
      value=predicates.Predicate(
        equals=predicates.Equals(
          key="id",
          value=metadata.MetadataValue(raw_string="123")
        )
      )
    )

    new_value = keyval.StoreValue(
      value={"status": metadata.MetadataValue(raw_string="published")}
    )

    response = await client.upsert(
      db_query.Upsert(
        store="test store",
        schema="analytics",
        condition=condition,
        new_value=new_value,
        merge_metadata=True
      )
    )

if __name__ == "__main__":
  asyncio.run(upsert())
```
</details>
