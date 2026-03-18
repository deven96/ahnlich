---
title: Convert to Embeddings
sidebar_position: 20
---

# Convert to Embeddings

The `convertStoreInputToEmbeddings` method converts raw inputs (text, images, audio) into embeddings using a specified AI model, without storing them in a database.

## Method Signature

```typescript
async convertStoreInputToEmbeddings(
  request: ConvertStoreInputToEmbeddingsRequest
): Promise<StoreInputToEmbeddingsList>
```

## Parameters

The `ConvertStoreInputToEmbeddingsRequest` contains:
- `storeInputs`: Array of inputs to convert (text, images, or audio)
- `preprocessAction`: How to preprocess inputs
- `model`: AI model to use for conversion
- `executionProvider` (optional): Execution provider (CPU, CUDA, etc.)
- `modelParams` (optional): Model-specific parameters

## Basic Example

```typescript
import { createPromiseClient } from "@connectrpc/connect";
import { createConnectTransport } from "@connectrpc/connect-node";
import { AiService } from "./grpc/services/ai_service_connect";
import { AiModel, PreprocessAction } from "./grpc/ai";

const transport = createConnectTransport({
  baseUrl: "http://localhost:1370",
  httpVersion: "2",
});

const client = createPromiseClient(AiService, transport);

const inputs = [
  { rawString: "Hello world" },
  { rawString: "Goodbye world" },
];

const response = await client.convertStoreInputToEmbeddings({
  storeInputs: inputs,
  preprocessAction: PreprocessAction.NO_PREPROCESSING,
  model: AiModel.ALL_MINI_LM_L6_V2,
});

// Access embeddings
for (const item of response.values) {
  if (item.variant.case === "single") {
    const embedding = item.variant.value.embedding;
    console.log(`Embedding size: ${embedding.key.length}`);
  }
}
```

## Face Detection with Metadata (Buffalo-L / SFace)

Starting from version 0.2.1, face detection models return **bounding box metadata** alongside embeddings:

```typescript
import { createPromiseClient } from "@connectrpc/connect";
import { createConnectTransport } from "@connectrpc/connect-node";
import { AiService } from "./grpc/services/ai_service_connect";
import { AiModel, PreprocessAction } from "./grpc/ai";
import * as fs from "fs";

const transport = createConnectTransport({
  baseUrl: "http://localhost:1370",
  httpVersion: "2",
});

const client = createPromiseClient(AiService, transport);

// Load image
const imageBytes = fs.readFileSync("group_photo.jpg");

const inputs = [{ image: imageBytes }];

const response = await client.convertStoreInputToEmbeddings({
  storeInputs: inputs,
  preprocessAction: PreprocessAction.MODEL_PREPROCESSING,
  model: AiModel.BUFFALO_L,
});

// Process each detected face
for (const item of response.values) {
  if (item.variant.case === "multiple") {
    const faces = item.variant.value.embeddings;
    console.log(`Detected ${faces.length} faces`);
    
    for (const faceData of faces) {
      // Access embedding
      const embedding = faceData.embedding!.key; // Float32Array
      console.log(`Embedding size: ${embedding.length}`);
      
      // Access bounding box metadata
      if (faceData.metadata) {
        const metadata = faceData.metadata.value;
        
        const bboxX1 = parseFloat(metadata.bbox_x1!.value!.value as string);
        const bboxY1 = parseFloat(metadata.bbox_y1!.value!.value as string);
        const bboxX2 = parseFloat(metadata.bbox_x2!.value!.value as string);
        const bboxY2 = parseFloat(metadata.bbox_y2!.value!.value as string);
        const confidence = parseFloat(metadata.confidence!.value!.value as string);
        
        console.log(`Face at (${bboxX1.toFixed(3)}, ${bboxY1.toFixed(3)}) ` +
                    `to (${bboxX2.toFixed(3)}, ${bboxY2.toFixed(3)})`);
        console.log(`Confidence: ${confidence.toFixed(3)}`);
      }
    }
  }
}
```

## Metadata Fields (Face Detection Models)

For Buffalo-L and SFace models, each detected face includes:

| Field | Type | Range | Description |
|-------|------|-------|-------------|
| `bbox_x1` | number | 0.0-1.0 | Normalized x-coordinate of top-left corner |
| `bbox_y1` | number | 0.0-1.0 | Normalized y-coordinate of top-left corner |
| `bbox_x2` | number | 0.0-1.0 | Normalized x-coordinate of bottom-right corner |
| `bbox_y2` | number | 0.0-1.0 | Normalized y-coordinate of bottom-right corner |
| `confidence` | number | 0.0-1.0 | Detection confidence score |

Coordinates are normalized to 0-1 range. To convert to pixel coordinates:
```typescript
import sharp from "sharp";

const img = sharp("photo.jpg");
const { width, height } = await img.metadata();

const pixelX1 = Math.round(bboxX1 * width!);
const pixelY1 = Math.round(bboxY1 * height!);
const pixelX2 = Math.round(bboxX2 * width!);
const pixelY2 = Math.round(bboxY2 * height!);
```

## Using Model Parameters

Face detection models support tuning via `modelParams`:

```typescript
const response = await client.convertStoreInputToEmbeddings({
  storeInputs: inputs,
  preprocessAction: PreprocessAction.MODEL_PREPROCESSING,
  model: AiModel.BUFFALO_L,
  modelParams: { 
    confidence_threshold: "0.9"  // Higher = fewer faces
  },
});
```

## Response Structure

```typescript
interface StoreInputToEmbeddingsList {
  values: SingleInputToEmbedding[];
}

interface SingleInputToEmbedding {
  input?: StoreInput;              // Original input
  variant: {
    case: "single" | "multiple";
    value: EmbeddingWithMetadata | MultipleEmbedding;
  };
}

interface EmbeddingWithMetadata {
  embedding?: StoreKey;            // The embedding vector
  metadata?: StoreValue;           // Optional metadata (e.g., bounding boxes)
}

interface MultipleEmbedding {
  embeddings: EmbeddingWithMetadata[];  // One per detected face
}
```

## Use Cases

- **Testing models**: Quickly test how different inputs are embedded
- **Batch processing**: Generate embeddings for analysis without storage
- **Face detection**: Extract face locations and embeddings from photos
- **Quality control**: Filter low-confidence detections before storage
- **Visualization**: Draw bounding boxes on images using metadata
