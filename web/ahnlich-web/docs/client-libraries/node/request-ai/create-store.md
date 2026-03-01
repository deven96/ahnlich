---
title: Create Store
sidebar_position: 5
---

# Create Store

The CreateStore request creates a new AI store with specified AI models.

* **Input**: Store name, query model, index model, optional predicates, and configuration flags.

* **Behavior**: Creates a new AI store that automatically generates embeddings using the specified models.

* **Response**: Confirmation of store creation.

<details>
  <summary>Click to expand source code</summary>

```ts
import { createAiClient } from "ahnlich-client-node";
import { CreateStore } from "ahnlich-client-node/grpc/ai/query_pb";
import { AIModel } from "ahnlich-client-node/grpc/ai/models_pb";

async function createStore() {
  const client = createAiClient("127.0.0.1:1370");

  await client.createStore(
    new CreateStore({
      store: "ai_store",
      queryModel: AIModel.ALL_MINI_LM_L6_V2,
      indexModel: AIModel.ALL_MINI_LM_L6_V2,
      predicates: ["brand", "category"],
      errorIfExists: true,
      storeOriginal: true,
    })
  );

  console.log("AI store created successfully");
}

createStore();
```
</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `store` | `string` | Yes | The name for the new AI store |
| `queryModel` | `AIModel` | Yes | AI model for embedding queries |
| `indexModel` | `AIModel` | Yes | AI model for embedding stored data |
| `predicates` | `string[]` | No | List of predicate keys to index |
| `errorIfExists` | `boolean` | No | If `true`, throws error if store exists |
| `storeOriginal` | `boolean` | No | If `true`, stores original input alongside embeddings |

## Available AI Models

| Model | Type | Description |
|-------|------|-------------|
| `AIModel.ALL_MINI_LM_L6_V2` | Text | Sentence transformer for text embeddings |
| `AIModel.RESNET50` | Image | Image classification and embeddings |
| `AIModel.CLIP_VIT_B32` | Multimodal | Text and image embeddings |
| `AIModel.BUFFALO_L` | Face | Face detection and recognition |
| `AIModel.SFACE_YUNET` | Face | Face detection with YuNet |

## Example with Image Model

<details>
  <summary>Click to expand source code</summary>

```ts
import { createAiClient } from "ahnlich-client-node";
import { CreateStore } from "ahnlich-client-node/grpc/ai/query_pb";
import { AIModel } from "ahnlich-client-node/grpc/ai/models_pb";

async function createImageStore() {
  const client = createAiClient("127.0.0.1:1370");

  await client.createStore(
    new CreateStore({
      store: "image_store",
      queryModel: AIModel.RESNET50,
      indexModel: AIModel.RESNET50,
      predicates: ["filename", "category"],
      errorIfExists: true,
      storeOriginal: true,
    })
  );

  console.log("Image store created successfully");
}

createImageStore();
```
</details>

## Notes

- The query and index models typically should be the same for consistent similarity
- `storeOriginal: true` is useful for retrieving the original text/image in results
- See [Type Meanings](/docs/client-libraries/node/type-meanings) for more details on AI models
