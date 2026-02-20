---
title: Get Key
---

# Get Key

The `GetKey` request retrieves specific entries from an AI store by their exact keys. This is a **direct lookup** operation that returns the stored data and metadata for the provided keys.

## Parameters

* `store` – Name of the AI store to query
* `keys` – List of `StoreInput` keys to retrieve (the original inputs used when storing)

## Behavior

- Performs exact key lookup (not similarity search)
- Returns entries with stored data and metadata for each found key
- Missing keys are silently skipped (no error for non-existent keys)
- Useful for retrieving known items you previously stored

<details>
  <summary>Click to expand source code</summary>

  ```py
  import asyncio
  from grpclib.client import Channel
  from ahnlich_client_py.grpc.services.ai_service import AiServiceStub
  from ahnlich_client_py.grpc.ai import query as ai_query
  from ahnlich_client_py.grpc import keyval


  async def get_key():
    async with Channel(host="127.0.0.1", port=1370) as channel:
        client = AiServiceStub(channel)
        
        response = await client.get_key(
            ai_query.GetKey(
                store="test store 1",
                keys=[
                    keyval.StoreInput(raw_string="Adidas Yeezy"),
                    keyval.StoreInput(raw_string="Nike Air Jordans"),
                ]
            )
        )
        
        # Response contains entries for each found key
        for entry in response.entries:
            print(f"Key: {entry.key}")
            print(f"Value: {entry.value}")


  if __name__ == "__main__":
    asyncio.run(get_key())
  ```
</details>

## Response

Returns a list of entries, where each entry contains:
- `key` – The original `StoreInput` that was stored
- `value` – The associated metadata (StoreValue)

If a requested key doesn't exist in the store, it won't appear in the results.
