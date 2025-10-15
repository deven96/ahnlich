---
title: Advanced
sidebar_position: 30
---

# Advanced

Unlike Ahnlich DB, which is concerned with similarity algorithms and indexing, **Ahnlich AI focuses on embedding generation**. The service introduces **model-aware stores**, where you define the embedding models used for both data insertion (indexing) and querying. This abstraction lets developers work directly with raw inputs (text or images) while the AI proxy handles embedding generation.

## Supported Models

Ahnlich AI includes several pre-trained models that can be configured depending on your workload. These cover both **text embeddings** and **image embeddings**:

| Model Name | Type | Description |
| ----- | ----- | ----- |
| ALL\_MINI\_LM\_L6\_V2 | Text | Lightweight sentence transformer. Fast and memory-efficient, ideal for semantic similarity in applications like FAQ search or chatbots. |
| ALL\_MINI\_LM\_L12\_V2 | Text | Larger variant of MiniLM. Higher accuracy for nuanced text similarity tasks, but with increased compute requirements. |
| BGE\_BASE\_EN\_V15 | Text | Base version of the BGE (English v1.5) model. Balanced performance and speed, suitable for production-scale applications. |
| BGE\_LARGE\_EN\_V15 | Text | High-accuracy embedding model for semantic search and retrieval. Best choice when precision is more important than latency. |
| RESNET50 | Image | Convolutional Neural Network (CNN) for extracting embeddings from images. Useful for content-based image retrieval and clustering. |
| CLIP\_VIT\_B32\_IMAGE | Image | Vision Transformer encoder from the CLIP model. Produces embeddings aligned with its paired text encoder for multimodal tasks. |
| CLIP\_VIT\_B32\_TEXT | Text | Text encoder from CLIP. Designed to map textual inputs into the same space as CLIP image embeddings for text-to-image or image-to-text search. |

## Supported Input Types

| Input Type | Description |
| ----- | ----- |
| RAW\_STRING | Accepts natural text (sentences, paragraphs). Transformed into embeddings via a selected text-based model. |
| IMAGE | Accepts image files as input. Converted into embeddings via a selected image-based model (e.g., ResNet or CLIP). |

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