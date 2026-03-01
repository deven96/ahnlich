---
title: Set
sidebar_position: 6
---

# Set

The Set request inserts entries into an AI store. The AI server automatically generates embeddings for the provided inputs.

* **Input**: Store name, array of entries (input-value pairs), and preprocessing options.

* **Behavior**: Generates embeddings for each input and stores them with associated metadata.

* **Response**: Confirmation of the operation.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createAiClient } from "ahnlich-client-node";
import { Set } from "ahnlich-client-node/grpc/ai/query_pb";
import { AiStoreEntry, StoreInput, StoreValue } from "ahnlich-client-node/grpc/keyval_pb";
import { MetadataValue } from "ahnlich-client-node/grpc/metadata_pb";
import { PreprocessAction } from "ahnlich-client-node/grpc/ai/preprocess_pb";

async function setEntries() {
  const client = createAiClient("127.0.0.1:1370");

  await client.set(
    new Set({
      store: "ai_store",
      inputs: [
        new AiStoreEntry({
          key: new StoreInput({ value: { case: "rawString", value: "Jordan One" } }),
          value: new StoreValue({
            value: {
              brand: new MetadataValue({ value: { case: "rawString", value: "Nike" } }),
            },
          }),
        }),
      ],
      preprocessAction: PreprocessAction.NO_PREPROCESSING,
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
| `store` | `string` | Yes | The name of the AI store |
| `inputs` | `AiStoreEntry[]` | Yes | Array of entries to insert |
| `preprocessAction` | `PreprocessAction` | No | Preprocessing to apply to inputs |

## AiStoreEntry Structure

| Field | Type | Description |
|-------|------|-------------|
| `key` | `StoreInput` | The input (text or binary) to embed |
| `value` | `StoreValue` | Metadata associated with the entry |

## StoreInput Types

| Type | Description |
|------|-------------|
| `rawString` | Text input for text models |
| `image` | Binary image data for image models |

## Example with Multiple Text Entries

<details>
  <summary>Click to expand source code</summary>

```ts
import { createAiClient } from "ahnlich-client-node";
import { Set } from "ahnlich-client-node/grpc/ai/query_pb";
import { AiStoreEntry, StoreInput, StoreValue } from "ahnlich-client-node/grpc/keyval_pb";
import { MetadataValue } from "ahnlich-client-node/grpc/metadata_pb";
import { PreprocessAction } from "ahnlich-client-node/grpc/ai/preprocess_pb";

async function setMultipleEntries() {
  const client = createAiClient("127.0.0.1:1370");

  const products = [
    { name: "Air Jordan 1", brand: "Nike", category: "Basketball" },
    { name: "Ultraboost", brand: "Adidas", category: "Running" },
    { name: "Chuck Taylor", brand: "Converse", category: "Casual" },
  ];

  await client.set(
    new Set({
      store: "products",
      inputs: products.map(
        (p) =>
          new AiStoreEntry({
            key: new StoreInput({ value: { case: "rawString", value: p.name } }),
            value: new StoreValue({
              value: {
                brand: new MetadataValue({ value: { case: "rawString", value: p.brand } }),
                category: new MetadataValue({ value: { case: "rawString", value: p.category } }),
              },
            }),
          })
      ),
      preprocessAction: PreprocessAction.NO_PREPROCESSING,
    })
  );
}

setMultipleEntries();
```
</details>

## Example with Binary Image

<details>
  <summary>Click to expand source code</summary>

```ts
import * as fs from "fs";
import { createAiClient } from "ahnlich-client-node";
import { Set } from "ahnlich-client-node/grpc/ai/query_pb";
import { AiStoreEntry, StoreInput, StoreValue } from "ahnlich-client-node/grpc/keyval_pb";
import { MetadataValue } from "ahnlich-client-node/grpc/metadata_pb";
import { PreprocessAction } from "ahnlich-client-node/grpc/ai/preprocess_pb";

async function setImageEntry() {
  const client = createAiClient("127.0.0.1:1370");

  const imageData = fs.readFileSync("./product.jpg");

  await client.set(
    new Set({
      store: "image_store",
      inputs: [
        new AiStoreEntry({
          key: new StoreInput({ value: { case: "image", value: imageData } }),
          value: new StoreValue({
            value: {
              filename: new MetadataValue({ value: { case: "rawString", value: "product.jpg" } }),
            },
          }),
        }),
      ],
      preprocessAction: PreprocessAction.NO_PREPROCESSING,
    })
  );
}

setImageEntry();
```
</details>
