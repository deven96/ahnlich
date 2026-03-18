---
title: Convert to Embeddings
sidebar_position: 20
---

# Convert to Embeddings

The `ConvertStoreInputToEmbeddings` RPC converts raw inputs (text, images, audio) into embeddings using a specified AI model, without storing them in a database.

## Method Signature

```go
ConvertStoreInputToEmbeddings(
    ctx context.Context,
    in *ConvertStoreInputToEmbeddings,
    opts ...grpc.CallOption,
) (*StoreInputToEmbeddingsList, error)
```

## Parameters

The `ConvertStoreInputToEmbeddings` request contains:
- `StoreInputs`: Slice of inputs to convert (text, images, or audio)
- `PreprocessAction`: How to preprocess inputs
- `Model`: AI model to use for conversion
- `ExecutionProvider` (optional): Execution provider (CPU, CUDA, etc.)
- `ModelParams` (optional): Model-specific parameters

## Basic Example

```go
package main

import (
    "context"
    "fmt"
    "log"
    
    "google.golang.org/grpc"
    "google.golang.org/grpc/credentials/insecure"
    
    pb "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/query"
    "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/models"
    "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/preprocess"
    "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/keyval"
    "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/services/ai_service"
)

func main() {
    conn, err := grpc.Dial("localhost:1370", grpc.WithTransportCredentials(insecure.NewCredentials()))
    if err != nil {
        log.Fatalf("Failed to connect: %v", err)
    }
    defer conn.Close()
    
    client := ai_service.NewAIServiceClient(conn)
    ctx := context.Background()
    
    inputs := []*keyval.StoreInput{
        {Value: &keyval.StoreInput_RawString{RawString: "Hello world"}},
        {Value: &keyval.StoreInput_RawString{RawString: "Goodbye world"}},
    }
    
    response, err := client.ConvertStoreInputToEmbeddings(ctx, &pb.ConvertStoreInputToEmbeddings{
        StoreInputs:       inputs,
        PreprocessAction:  preprocess.PreprocessAction_NoPreprocessing,
        Model:             models.AiModel_ALL_MINI_LM_L6_V2,
    })
    if err != nil {
        log.Fatalf("Failed to convert: %v", err)
    }
    
    // Access embeddings
    for _, item := range response.Values {
        if single := item.GetSingle(); single != nil {
            if embedding := single.GetEmbedding(); embedding != nil {
                fmt.Printf("Embedding size: %d\n", len(embedding.Key))
            }
        }
    }
}
```

## Face Detection with Metadata (Buffalo-L / SFace)

Starting from version 0.2.2, face detection models return **bounding box metadata** alongside embeddings:

```go
package main

import (
    "context"
    "fmt"
    "log"
    "os"
    "strconv"
    
    "google.golang.org/grpc"
    "google.golang.org/grpc/credentials/insecure"
    
    pb "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/query"
    "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/models"
    "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/preprocess"
    "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/keyval"
    "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/services/ai_service"
)

func main() {
    conn, err := grpc.Dial("localhost:1370", grpc.WithTransportCredentials(insecure.NewCredentials()))
    if err != nil {
        log.Fatalf("Failed to connect: %v", err)
    }
    defer conn.Close()
    
    client := ai_service.NewAIServiceClient(conn)
    ctx := context.Background()
    
    // Load image
    imageBytes, err := os.ReadFile("group_photo.jpg")
    if err != nil {
        log.Fatalf("Failed to read image: %v", err)
    }
    
    inputs := []*keyval.StoreInput{
        {Value: &keyval.StoreInput_Image{Image: imageBytes}},
    }
    
    response, err := client.ConvertStoreInputToEmbeddings(ctx, &pb.ConvertStoreInputToEmbeddings{
        StoreInputs:       inputs,
        PreprocessAction:  preprocess.PreprocessAction_ModelPreprocessing,
        Model:             models.AiModel_BUFFALO_L,
    })
    if err != nil {
        log.Fatalf("Failed to convert: %v", err)
    }
    
    // Process each detected face
    for _, item := range response.Values {
        if multiple := item.GetMultiple(); multiple != nil {
            fmt.Printf("Detected %d faces\n", len(multiple.Embeddings))
            
            for _, faceData := range multiple.Embeddings {
                // Access embedding
                if embedding := faceData.GetEmbedding(); embedding != nil {
                    fmt.Printf("Embedding size: %d\n", len(embedding.Key))
                }
                
                // Access bounding box metadata
                if metadata := faceData.GetMetadata(); metadata != nil {
                    if bboxX1, ok := metadata.Value["bbox_x1"]; ok {
                        if rawStr := bboxX1.GetValue().(*keyval.MetadataValue_RawString); rawStr != nil {
                            x1, _ := strconv.ParseFloat(rawStr.RawString, 32)
                            fmt.Printf("Face bbox x1: %.3f\n", x1)
                        }
                    }
                    
                    if confidence, ok := metadata.Value["confidence"]; ok {
                        if rawStr := confidence.GetValue().(*keyval.MetadataValue_RawString); rawStr != nil {
                            conf, _ := strconv.ParseFloat(rawStr.RawString, 32)
                            fmt.Printf("Confidence: %.3f\n", conf)
                        }
                    }
                }
            }
        }
    }
}
```

## Metadata Fields (Face Detection Models)

For Buffalo-L and SFace models, each detected face includes:

| Field | Type | Range | Description |
|-------|------|-------|-------------|
| `bbox_x1` | float32 | 0.0-1.0 | Normalized x-coordinate of top-left corner |
| `bbox_y1` | float32 | 0.0-1.0 | Normalized y-coordinate of top-left corner |
| `bbox_x2` | float32 | 0.0-1.0 | Normalized x-coordinate of bottom-right corner |
| `bbox_y2` | float32 | 0.0-1.0 | Normalized y-coordinate of bottom-right corner |
| `confidence` | float32 | 0.0-1.0 | Detection confidence score |

Coordinates are normalized to 0-1 range. To convert to pixel coordinates:
```go
import "image"

img, _, err := image.Decode(file)
bounds := img.Bounds()
width := bounds.Dx()
height := bounds.Dy()

pixelX1 := int(bboxX1 * float32(width))
pixelY1 := int(bboxY1 * float32(height))
```

## Using Model Parameters

Face detection models support tuning via `ModelParams`:

```go
response, err := client.ConvertStoreInputToEmbeddings(ctx, &pb.ConvertStoreInputToEmbeddings{
    StoreInputs:       inputs,
    PreprocessAction:  preprocess.PreprocessAction_ModelPreprocessing,
    Model:             models.AiModel_BUFFALO_L,
    ModelParams: map[string]string{
        "confidence_threshold": "0.9", // Higher = fewer faces
    },
})
```

## Response Structure

```go
type StoreInputToEmbeddingsList struct {
    Values []*SingleInputToEmbedding
}

type SingleInputToEmbedding struct {
    Input   *StoreInput
    // Variant is one of:
    // - *SingleInputToEmbedding_Single (for text/image models)
    // - *SingleInputToEmbedding_Multiple (for face detection)
}

type EmbeddingWithMetadata struct {
    Embedding *StoreKey            // The embedding vector
    Metadata  *StoreValue          // Optional metadata (e.g., bounding boxes)
}

type MultipleEmbedding struct {
    Embeddings []*EmbeddingWithMetadata  // One per detected face
}
```

## Use Cases

- **Testing models**: Quickly test how different inputs are embedded
- **Batch processing**: Generate embeddings for analysis without storage
- **Face detection**: Extract face locations and embeddings from photos
- **Quality control**: Filter low-confidence detections before storage
- **Visualization**: Draw bounding boxes on images using metadata
