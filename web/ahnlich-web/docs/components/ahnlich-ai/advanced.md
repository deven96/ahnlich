---
title: Advanced
sidebar_position: 30
---

# Advanced

Unlike Ahnlich DB, which is concerned with similarity algorithms and indexing, **Ahnlich AI focuses on embedding generation**. The service introduces **model-aware stores**, where you define the embedding models used for both data insertion (indexing) and querying. This abstraction lets developers work directly with raw inputs (text or images) while the AI proxy handles embedding generation.

## Supported Models

Ahnlich AI includes several pre-trained models that can be configured depending on your workload. These cover both **text embeddings** and **image embeddings**:

| Model Name | String Name | Type | Max Input | Embedding Dim | Description |
| ----- | ----- | ----- | ----- | ----- | ----- |
| ALL\_MINI\_LM\_L6\_V2 | all-minilm-l6-v2 | Text | 256 tokens | 384 | Lightweight sentence transformer. Fast and memory-efficient, ideal for semantic similarity in applications like FAQ search or chatbots. |
| ALL\_MINI\_LM\_L12\_V2 | all-minilm-l12-v2 | Text | 256 tokens | 384 | Larger variant of MiniLM. Higher accuracy for nuanced text similarity tasks, but with increased compute requirements. |
| BGE\_BASE\_EN\_V15 |  bge-base-en-v1.5 | Text | 512 tokens | 768 | Base version of the BGE (English v1.5) model. Balanced performance and speed, suitable for production-scale applications. |
| BGE\_LARGE\_EN\_V15 | bge-large-en-v1.5 | Text | 512 tokens | 1024 | High-accuracy embedding model for semantic search and retrieval. Best choice when precision is more important than latency. |
| RESNET50 | resnet-50 | Image | 224x224 px | 2048 | Convolutional Neural Network (CNN) for extracting embeddings from images. Useful for content-based image retrieval and clustering. |
| CLIP\_VIT\_B32\_IMAGE | clip-vit-b32-image | Image | 224x224 px | 512 | Vision Transformer encoder from the CLIP model. Produces embeddings aligned with its paired text encoder for multimodal tasks. |
| CLIP\_VIT\_B32\_TEXT | clip-vit-b32-text | Text | 77 tokens | 512 | Text encoder from CLIP. Designed to map textual inputs into the same space as CLIP image embeddings for text-to-image or image-to-text search. |
| BUFFALO\_L | buffalo-l | Image (Face) | 640x640 px | 512 | Face detection and recognition model. Detects faces in images and generates embeddings for each detected face. **Non-commercial use only.** |
| SFACE\_YUNET | sface-yunet | Image (Face) | 640x640 px | 128 | Lightweight face detection (YuNet) + recognition (SFace) pipeline. Apache 2.0 / MIT licensed - commercially usable. |
| CLAP\_AUDIO | clap-audio | Audio | 10 sec max | 512 | Audio encoder from the CLAP model. Produces embeddings from audio inputs for audio similarity search and audio-to-text retrieval. |
| CLAP\_TEXT | clap-text | Text | 512 tokens | 512 | Text encoder from the CLAP model. Maps textual descriptions into the same embedding space as CLAP audio embeddings for text-to-audio search. |

## Model Constraints

### Audio Models (CLAP)

| Constraint | Value | Notes |
| ----- | ----- | ----- |
| Max duration | 10 seconds | Longer clips will error with `AudioTooLongError` |
| Sample rate | 48 kHz | Audio is automatically resampled |
| Max samples | 480,000 | 48,000 Hz × 10 seconds |
| Preprocessing | Required | `NoPreprocessing` not supported - always use `ModelPreprocessing` |

### Face Models (Buffalo\_L, SFace+YuNet)

| Constraint | Value | Notes |
| ----- | ----- | ----- |
| Input size | 640x640 px | Images are resized internally |
| Face alignment | 112x112 px | Standard ArcFace alignment |
| Embedding mode | OneToMany | Returns one embedding per detected face |
| Preprocessing | Required | `NoPreprocessing` not supported |
| Query constraint | Single face | Query images must contain exactly 1 face |

### Cross-Modal Compatibility

| Model Pair | Shared Dim | Use Case |
| ----- | ----- | ----- |
| `clip-vit-b32-text` + `clip-vit-b32-image` | 512 | Text-to-image / image-to-text search |
| `clap-text` + `clap-audio` | 512 | Text-to-audio / audio-to-text search |

## Supported Input Types

| Input Type | Description |
| ----- | ----- |
| RAW\_STRING | Accepts natural text (sentences, paragraphs). Transformed into embeddings via a selected text-based model. |
| IMAGE | Accepts image files as input. Converted into embeddings via a selected image-based model (e.g., ResNet or CLIP). |
| AUDIO | Accepts audio data as input. Converted into embeddings via an audio-based model (e.g., CLAP Audio). |

## Example – Creating a Model-Aware Store

```
CREATESTORE my_store QUERYMODEL all-minilm-l6-v2 INDEXMODEL all-minilm-l6-v2
```

- **index_model** - defines how inserted data is embedded before being stored in Ahnlich DB.

- **query_model** - defines how queries are embedded at search time.

- Both models must output embeddings of the **same dimensionality** to ensure compatibility.

## Choosing the Right Model

| Model | Best Use Case |
| ----- | ----- |
| MiniLM (L6/L12) | Fast, efficient semantic similarity (FAQs, chatbots). |
| BGE (Base/Large) | High semantic accuracy for production-scale applications. |
| ResNet50 | Image-to-image similarity and clustering. |
| CLIP (Text+Image) | Multimodal retrieval (text-to-image / image-to-text search). |
| Buffalo\_L | Face detection and recognition in images (e.g., group photos, ID verification). |
| SFace+YuNet | Lightweight face detection and recognition (e.g., real-time face matching). |
| CLAP (Audio+Text) | Audio similarity search and text-to-audio retrieval. |

## Model Parameters (`model_params`)

Some AI models accept optional runtime parameters via `model_params` — a `map<string, string>` field available on `Set`, `GetSimN`, and `ConvertStoreInputToEmbeddings` requests. These parameters let you tune model behavior at inference time without changing store configuration.

When `model_params` is empty (or omitted), models use their built-in defaults. Models that don't support any parameters simply ignore the field.

### Supported Parameters by Model

| Model | Parameter | Type | Default | Description |
| ----- | ----- | ----- | ----- | ----- |
| **Buffalo\_L** | `confidence_threshold` | float (0.0–1.0) | `0.5` | Minimum detection confidence for a face to be included. Higher values = fewer but more confident detections. |
| **Buffalo\_L** | `attributes` | string (comma-separated) | (empty) | Optional attributes to compute. Use `genderage` to enable age and gender predictions. When omitted, only face embeddings and bounding boxes are computed. |
| **SFace+YuNet** | `confidence_threshold` | float (0.0–1.0) | `0.6` | Minimum detection confidence for a face to be included. Higher values = fewer but more confident detections. |

Text embedding models (MiniLM, BGE), image models (ResNet, CLIP), and audio models (CLAP) do not currently use `model_params`.

### Usage Examples

**Rust** — setting a high confidence threshold for face detection:
```rust
use std::collections::HashMap;

let mut model_params = HashMap::new();
model_params.insert("confidence_threshold".to_string(), "0.9".to_string());

let set_params = Set {
    store: "faces_store".to_string(),
    inputs: vec![/* ... */],
    preprocess_action: PreprocessAction::NoPreprocessing as i32,
    execution_provider: None,
    model_params,
};
```

**Python** — using default parameters (empty dict):
```python
await client.set(
    ai_query.Set(
        store="faces_store",
        inputs=[...],
        preprocess_action=preprocess.PreprocessAction.NoPreprocessing,
        model_params={}  # uses model defaults
    )
)
```

**Python** — custom confidence threshold:
```python
await client.set(
    ai_query.Set(
        store="faces_store",
        inputs=[...],
        preprocess_action=preprocess.PreprocessAction.NoPreprocessing,
        model_params={"confidence_threshold": "0.9"}
    )
)
```

### When to Tune `model_params`

- **Inclusive detection** (e.g., group photos where you want all faces): Use a lower threshold like `0.3`
- **Standard detection** (balanced): Use the model default (`0.5` for Buffalo\_L, `0.6` for SFace+YuNet)
- **Strict detection** (e.g., ID verification where only clear faces matter): Use a higher threshold like `0.9`

## Embedding Metadata

Starting from version 0.2.2, face detection models (Buffalo\_L and SFace+YuNet) return **bounding box metadata** alongside embeddings. This allows you to access face location and confidence information without re-running detection.

### Metadata Fields (Face Detection Models)

For each detected face, the following metadata is automatically included:

| Field | Type | Range | Description |
| ----- | ----- | ----- | ----- |
| `bbox_x1` | float | 0.0–1.0 | Normalized x-coordinate of top-left corner |
| `bbox_y1` | float | 0.0–1.0 | Normalized y-coordinate of top-left corner |
| `bbox_x2` | float | 0.0–1.0 | Normalized x-coordinate of bottom-right corner |
| `bbox_y2` | float | 0.0–1.0 | Normalized y-coordinate of bottom-right corner |
| `confidence` | float | 0.0–1.0 | Detection confidence score |

**Buffalo\_L only** — the following fields are included when `attributes=genderage` is specified:

| Field | Type | Range | Description |
| ----- | ----- | ----- | ----- |
| `gender_female_prob` | float | 0.0–1.0 | Probability of female gender |
| `gender_male_prob` | float | 0.0–1.0 | Probability of male gender |
| `age` | float | 0.0–100.0 | Predicted age in years |

**Coordinates are normalized** to the 0-1 range, making them independent of the original image resolution. To convert to pixel coordinates, multiply by the image width/height:

```
pixel_x1 = bbox_x1 * image_width
pixel_y1 = bbox_y1 * image_height
```

### Metadata Storage

When you insert images using face detection models:
- Embeddings are stored in Ahnlich DB as usual
- Metadata (bounding boxes, confidence) is merged into the `StoreValue` for each face
- Metadata is returned in `GetSimN`, `GetPred`, and `ConvertStoreInputToEmbeddings` responses

### API Response Structure

The `ConvertStoreInputToEmbeddings` API returns `EmbeddingWithMetadata` for face models:

```protobuf
message EmbeddingWithMetadata {
  keyval.StoreKey embedding = 1;           // The face embedding vector
  optional keyval.StoreValue metadata = 2; // Bounding box + confidence
}
```

For OneToMany models (face detection), multiple `EmbeddingWithMetadata` objects are returned—one per detected face.

### Usage Examples

**Rust** — accessing bounding box metadata:
```rust
use ahnlich_client_rs::prelude::*;

let response = client.convert_to_embeddings(
    store_name,
    vec![StoreInput::Image(image_bytes)],
    PreprocessAction::ModelPreprocessing,
    None,
    HashMap::new(),
).await?;

// For face detection models, variant is OneToMany
if let Some(Variant::Multiple(multi)) = &response.values[0].variant {
    for face in &multi.embeddings {
        if let Some(embedding) = &face.embedding {
            println!("Embedding dimensions: {}", embedding.key.len());
        }
        
        if let Some(metadata) = &face.metadata {
            let bbox_x1 = metadata.value.get("bbox_x1").unwrap();
            let bbox_y1 = metadata.value.get("bbox_y1").unwrap();
            let confidence = metadata.value.get("confidence").unwrap();
            
            println!("Face at ({}, {}) with confidence {}", 
                     bbox_x1, bbox_y1, confidence);
        }
    }
}
```

**Python** — accessing bounding box metadata:
```python
from ahnlich_client_py import AhnlichAIClient

response = await client.convert_store_input_to_embeddings(
    store="faces_store",
    inputs=[image_bytes],
    preprocess_action=PreprocessAction.ModelPreprocessing,
)

# Each face has embedding + metadata
for face_data in response.values[0].multiple.embeddings:
    embedding = face_data.embedding.key  # 512-dim vector for Buffalo_L
    metadata = face_data.metadata.value
    
    bbox_x1 = float(metadata["bbox_x1"].value)
    bbox_y1 = float(metadata["bbox_y1"].value)
    confidence = float(metadata["confidence"].value)
    
    print(f"Face at ({bbox_x1}, {bbox_y1}) with confidence {confidence}")
```

**TypeScript** — accessing bounding box metadata:
```typescript
import { AhnlichAIClient } from '@deven96/ahnlich-client-node';

const response = await client.convertStoreInputToEmbeddings({
  store: "faces_store",
  inputs: [{ image: imageBytes }],
  preprocessAction: PreprocessAction.MODEL_PREPROCESSING,
});

// Each detected face has embedding + metadata
for (const faceData of response.values[0].multiple.embeddings) {
  const embedding = faceData.embedding.key; // Float32Array
  const metadata = faceData.metadata.value;
  
  const bboxX1 = parseFloat(metadata.bbox_x1.value);
  const bboxY1 = parseFloat(metadata.bbox_y1.value);
  const confidence = parseFloat(metadata.confidence.value);
  
  console.log(`Face at (${bboxX1}, ${bboxY1}) with confidence ${confidence}`);
}
```

### Gender and Age Predictions (Buffalo_L)

Buffalo_L can compute **age and gender predictions** for each detected face by setting `attributes=genderage` in `model_params`. This adds three additional metadata fields per face: `gender_female_prob`, `gender_male_prob`, and `age`.

**Rust** — enabling gender and age predictions:
```rust
use std::collections::HashMap;
use ahnlich_client_rs::prelude::*;

let mut model_params = HashMap::new();
model_params.insert("attributes".to_string(), "genderage".to_string());

let response = client.convert_to_embeddings(
    store_name,
    vec![StoreInput::Image(image_bytes)],
    PreprocessAction::ModelPreprocessing,
    None,
    model_params,
).await?;

// Access gender/age metadata
if let Some(Variant::Multiple(multi)) = &response.values[0].variant {
    for face in &multi.embeddings {
        if let Some(metadata) = &face.metadata {
            let female_prob = metadata.value.get("gender_female_prob").unwrap();
            let male_prob = metadata.value.get("gender_male_prob").unwrap();
            let age = metadata.value.get("age").unwrap();
            
            println!("Age: {}, Female: {}, Male: {}", age, female_prob, male_prob);
        }
    }
}
```

**Python** — enabling gender and age predictions:
```python
from ahnlich_client_py import AhnlichAIClient

response = await client.convert_store_input_to_embeddings(
    store="faces_store",
    inputs=[image_bytes],
    preprocess_action=PreprocessAction.ModelPreprocessing,
    model_params={"attributes": "genderage"}
)

# Access gender/age metadata
for face_data in response.values[0].multiple.embeddings:
    metadata = face_data.metadata.value
    
    female_prob = float(metadata["gender_female_prob"].value)
    male_prob = float(metadata["gender_male_prob"].value)
    age = float(metadata["age"].value)
    
    print(f"Age: {age}, Female: {female_prob}, Male: {male_prob}")
```

**TypeScript** — enabling gender and age predictions:
```typescript
import { AhnlichAIClient } from '@deven96/ahnlich-client-node';

const response = await client.convertStoreInputToEmbeddings({
  store: "faces_store",
  inputs: [{ image: imageBytes }],
  preprocessAction: PreprocessAction.MODEL_PREPROCESSING,
  modelParams: { attributes: "genderage" }
});

// Access gender/age metadata
for (const faceData of response.values[0].multiple.embeddings) {
  const metadata = faceData.metadata.value;
  
  const femaleProb = parseFloat(metadata.gender_female_prob.value);
  const maleProb = parseFloat(metadata.gender_male_prob.value);
  const age = parseFloat(metadata.age.value);
  
  console.log(`Age: ${age}, Female: ${femaleProb}, Male: ${maleProb}`);
}
```

### Use Cases for Metadata

- **Face cropping**: Use bounding boxes to extract face regions from original images
- **Visualization**: Draw bounding boxes on images to show detected faces
- **Quality filtering**: Filter results by confidence score (e.g., only faces with confidence > 0.8)
- **Spatial queries**: Find faces in specific image regions (e.g., "faces in the top-left quadrant")
- **Deduplication**: Identify overlapping detections using bounding box coordinates
- **Demographic analysis** (Buffalo_L with `attributes=genderage`):
  - Age-based filtering (e.g., "find faces that appear under 18")
  - Gender distribution analysis in group photos
  - Age group clustering (children, adults, elderly)
  - Demographic insights for audience analysis

### Models Without Metadata

Text and image embedding models (MiniLM, BGE, ResNet, CLIP) do **not** return metadata. The `metadata` field will be `None` or empty for these models.