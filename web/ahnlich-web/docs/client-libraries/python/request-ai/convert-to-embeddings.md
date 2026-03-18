---
title: Convert to Embeddings
sidebar_position: 20
---

# Convert to Embeddings

The `convert_store_input_to_embeddings` method converts raw inputs (text, images, audio) into embeddings using a specified AI model, without storing them in a database.

## Method Signature

```python
async def convert_store_input_to_embeddings(
    self,
    request: ConvertStoreInputToEmbeddings
) -> StoreInputToEmbeddingsList
```

## Parameters

The `ConvertStoreInputToEmbeddings` request contains:
- `store_inputs`: List of inputs to convert (text, images, or audio)
- `preprocess_action`: How to preprocess inputs
- `model`: AI model to use for conversion
- `execution_provider` (optional): Execution provider (CPU, CUDA, etc.)
- `model_params` (optional): Model-specific parameters

## Basic Example

```python
from grpclib.client import Channel
from ahnlich_client_py.grpc.services.ai_service import AiServiceStub
from ahnlich_client_py.grpc.ai import query as ai_query
from ahnlich_client_py.grpc.ai.models import AiModel
from ahnlich_client_py.grpc.ai import preprocess
from ahnlich_client_py.grpc import keyval

async with Channel(host="127.0.0.1", port=1370) as channel:
    client = AiServiceStub(channel)
    
    inputs = [
        keyval.StoreInput(raw_string="Hello world"),
        keyval.StoreInput(raw_string="Goodbye world"),
    ]
    
    response = await client.convert_store_input_to_embeddings(
        ai_query.ConvertStoreInputToEmbeddings(
            store_inputs=inputs,
            preprocess_action=preprocess.PreprocessAction.NoPreprocessing,
            model=AiModel.ALL_MINI_LM_L6_V2,
        )
    )
    
    # Access embeddings
    for item in response.values:
        if item.single:
            print(f"Embedding size: {len(item.single.embedding.key)}")
```

## Face Detection with Metadata (Buffalo-L / SFace)

Starting from version 0.2.1, face detection models return **bounding box metadata** alongside embeddings:

```python
from grpclib.client import Channel
from ahnlich_client_py.grpc.services.ai_service import AiServiceStub
from ahnlich_client_py.grpc.ai import query as ai_query
from ahnlich_client_py.grpc.ai.models import AiModel
from ahnlich_client_py.grpc.ai import preprocess
from ahnlich_client_py.grpc import keyval

async with Channel(host="127.0.0.1", port=1370) as channel:
    client = AiServiceStub(channel)
    
    # Load image
    with open("group_photo.jpg", "rb") as f:
        image_bytes = f.read()
    
    inputs = [keyval.StoreInput(image=image_bytes)]
    
    response = await client.convert_store_input_to_embeddings(
        ai_query.ConvertStoreInputToEmbeddings(
            store_inputs=inputs,
            preprocess_action=preprocess.PreprocessAction.ModelPreprocessing,
            model=AiModel.BUFFALO_L,
        )
    )
    
    # Process each detected face
    for item in response.values:
        if item.multiple:
            print(f"Detected {len(item.multiple.embeddings)} faces")
            
            for face_data in item.multiple.embeddings:
                # Access embedding
                embedding = face_data.embedding.key  # 512-dim for Buffalo_L
                print(f"Embedding size: {len(embedding)}")
                
                # Access bounding box metadata
                if face_data.metadata:
                    metadata = face_data.metadata.value
                    
                    bbox_x1 = float(metadata["bbox_x1"].value)
                    bbox_y1 = float(metadata["bbox_y1"].value)
                    bbox_x2 = float(metadata["bbox_x2"].value)
                    bbox_y2 = float(metadata["bbox_y2"].value)
                    confidence = float(metadata["confidence"].value)
                    
                    print(f"Face at ({bbox_x1:.3f}, {bbox_y1:.3f}) "
                          f"to ({bbox_x2:.3f}, {bbox_y2:.3f})")
                    print(f"Confidence: {confidence:.3f}")
```

## Metadata Fields (Face Detection Models)

For Buffalo-L and SFace models, each detected face includes:

| Field | Type | Range | Description |
|-------|------|-------|-------------|
| `bbox_x1` | float | 0.0-1.0 | Normalized x-coordinate of top-left corner |
| `bbox_y1` | float | 0.0-1.0 | Normalized y-coordinate of top-left corner |
| `bbox_x2` | float | 0.0-1.0 | Normalized x-coordinate of bottom-right corner |
| `bbox_y2` | float | 0.0-1.0 | Normalized y-coordinate of bottom-right corner |
| `confidence` | float | 0.0-1.0 | Detection confidence score |

Coordinates are normalized to 0-1 range. To convert to pixel coordinates:
```python
from PIL import Image

img = Image.open("photo.jpg")
width, height = img.size

pixel_x1 = int(bbox_x1 * width)
pixel_y1 = int(bbox_y1 * height)
pixel_x2 = int(bbox_x2 * width)
pixel_y2 = int(bbox_y2 * height)
```

## Using Model Parameters

Face detection models support tuning via `model_params`:

```python
response = await client.convert_store_input_to_embeddings(
    ai_query.ConvertStoreInputToEmbeddings(
        store_inputs=inputs,
        preprocess_action=preprocess.PreprocessAction.ModelPreprocessing,
        model=AiModel.BUFFALO_L,
        model_params={"confidence_threshold": "0.9"},  # Higher = fewer faces
    )
)
```

## Response Structure

```python
class StoreInputToEmbeddingsList:
    values: List[SingleInputToEmbedding]

class SingleInputToEmbedding:
    input: StoreInput              # Original input
    single: EmbeddingWithMetadata  # For text/image models
    multiple: MultipleEmbedding    # For face detection models

class EmbeddingWithMetadata:
    embedding: StoreKey            # The embedding vector
    metadata: StoreValue           # Optional metadata (e.g., bounding boxes)

class MultipleEmbedding:
    embeddings: List[EmbeddingWithMetadata]  # One per detected face
```

## Use Cases

- **Testing models**: Quickly test how different inputs are embedded
- **Batch processing**: Generate embeddings for analysis without storage
- **Face detection**: Extract face locations and embeddings from photos
- **Quality control**: Filter low-confidence detections before storage
- **Visualization**: Draw bounding boxes on images using metadata
