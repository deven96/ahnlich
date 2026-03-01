---
title: Set
sidebar_position: 7
---

# Set

The Set request inserts or updates entries in a store. Each entry consists of a vector key and associated metadata value.

* **Input**: Store name and array of entries (key-value pairs).

* **Behavior**: Inserts new entries or updates existing ones if the key already exists.

* **Response**: Confirmation of the operation.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { Set } from "ahnlich-client-node/grpc/db/query_pb";
import { DbStoreEntry, StoreKey, StoreValue } from "ahnlich-client-node/grpc/keyval_pb";
import { MetadataValue } from "ahnlich-client-node/grpc/metadata_pb";

async function setEntries() {
  const client = createDbClient("127.0.0.1:1369");

  await client.set(
    new Set({
      store: "my_store",
      inputs: [
        new DbStoreEntry({
          key: new StoreKey({ key: [1.0, 2.0, 3.0, 4.0] }),
          value: new StoreValue({
            value: {
              label: new MetadataValue({ value: { case: "rawString", value: "A" } }),
            },
          }),
        }),
      ],
    })
  );

  console.log("Entry inserted successfully");
}

setEntries();
```
</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `store` | `string` | Yes | The name of the store |
| `inputs` | `DbStoreEntry[]` | Yes | Array of entries to insert/update |

## DbStoreEntry Structure

Each entry consists of:

| Field | Type | Description |
|-------|------|-------------|
| `key` | `StoreKey` | The vector key (must match store dimension) |
| `value` | `StoreValue` | Metadata associated with the vector |

## Example with Multiple Entries

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { Set } from "ahnlich-client-node/grpc/db/query_pb";
import { DbStoreEntry, StoreKey, StoreValue } from "ahnlich-client-node/grpc/keyval_pb";
import { MetadataValue } from "ahnlich-client-node/grpc/metadata_pb";

async function setMultipleEntries() {
  const client = createDbClient("127.0.0.1:1369");

  await client.set(
    new Set({
      store: "my_store",
      inputs: [
        new DbStoreEntry({
          key: new StoreKey({ key: [1.0, 2.0, 3.0, 4.0] }),
          value: new StoreValue({
            value: {
              label: new MetadataValue({ value: { case: "rawString", value: "First" } }),
              category: new MetadataValue({ value: { case: "rawString", value: "A" } }),
            },
          }),
        }),
        new DbStoreEntry({
          key: new StoreKey({ key: [5.0, 6.0, 7.0, 8.0] }),
          value: new StoreValue({
            value: {
              label: new MetadataValue({ value: { case: "rawString", value: "Second" } }),
              category: new MetadataValue({ value: { case: "rawString", value: "B" } }),
            },
          }),
        }),
      ],
    })
  );
}

setMultipleEntries();
```
</details>

## Example with Binary Metadata

<details>
  <summary>Click to expand source code</summary>

```ts
import { createDbClient } from "ahnlich-client-node";
import { Set } from "ahnlich-client-node/grpc/db/query_pb";
import { DbStoreEntry, StoreKey, StoreValue } from "ahnlich-client-node/grpc/keyval_pb";
import { MetadataValue } from "ahnlich-client-node/grpc/metadata_pb";

async function setWithBinaryMetadata() {
  const client = createDbClient("127.0.0.1:1369");

  const binaryData = new Uint8Array([0x01, 0x02, 0x03, 0x04]);

  await client.set(
    new Set({
      store: "my_store",
      inputs: [
        new DbStoreEntry({
          key: new StoreKey({ key: [1.0, 2.0, 3.0, 4.0] }),
          value: new StoreValue({
            value: {
              label: new MetadataValue({ value: { case: "rawString", value: "with binary" } }),
              image: new MetadataValue({ value: { case: "image", value: binaryData } }),
            },
          }),
        }),
      ],
    })
  );
}

setWithBinaryMetadata();
```
</details>
