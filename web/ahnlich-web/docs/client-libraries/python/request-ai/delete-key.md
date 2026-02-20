---
title: Delete Key
---

# Delete Key

How to delete keys and their associated values from a store using the Ahnlich AI Client.

The `Delete Key` operation removes one or more entries from a store. Each key uniquely identifies a vector-value pair in the store, and deleting it permanently removes both the key and the stored value.

This operation is useful when:

* You want to remove outdated or irrelevant data.

* You need to clean up test data.

* You want to maintain store integrity by pruning unused keys.

If the specified key does not exist, the behavior depends on the server configuration. In general, no changes are made, and the request safely returns without errors.

<details>
  <summary>Click to expand source code</summary>

  ```py
  import asyncio
  from grpclib.client import Channel
  from ahnlich_client_py.grpc.services.ai_service import AiServiceStub
  from ahnlich_client_py.grpc.ai import query as ai_query
  from ahnlich_client_py.grpc import keyval


  async def drop_key():
    async with Channel(host="127.0.0.1", port=1370) as channel:
        client = AiServiceStub(channel)
        response = await client.del_key(
            ai_query.DelKey(
                store="test store 1",
                keys=[keyval.StoreInput(raw_string="Custom Made Jordan 4")]
            )
        )
        print(response) # Del()


  if __name__ == "__main__":
    asyncio.run(drop_key())
  ```
</details>

## Behavior

* **Key match found** → The key and its associated value are permanently removed.

* **Key not found** → No action is performed; the request completes without altering the store.

## Source Code Example

In the context of the rest of the application code:

<details>
  <summary>Click to expand code</summary>
  
  ```py
  import asyncio
  from grpclib.client import Channel
  from ahnlich_client_py.grpc.services.ai_service import AiServiceStub
  from ahnlich_client_py.grpc.ai import query as ai_query


  async def trace_id():
    async with Channel(host="127.0.0.1", port=1370) as channel:
        client = AiServiceStub(channel)
        
        # Prepare tracing metadata
        tracing_id = "00-80e1afed08e019fc1110464cfa66635c-7a085853722dc6d2-01"
        metadata = {"ahnlich-trace-id": tracing_id}
        
        # Make request with metadata
        response = await client.ping(
            ai_query.Ping(),
            metadata=metadata
        )
        print(response) # Pong()


  if __name__ == "__main__":
    asyncio.run(trace_id())
  ```
</details>

