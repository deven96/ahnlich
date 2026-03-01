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