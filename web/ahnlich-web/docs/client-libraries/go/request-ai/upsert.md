---
title: Upsert
---

# Upsert

## Schema

This request accepts an optional `schema` field. When it is omitted, the server uses the `public` schema. Set `schema` to target a store in another schema.

## Description

The `Upsert` request updates a single entry matching a predicate condition in an AI store. The AI proxy automatically merges metadata and can re-embed new inputs.

Fields:
- `Store` - store name
- `Condition` - predicate that must match exactly one entry
- `NewInput` (optional) - new raw input (text, image, or audio) to re-embed
- `NewValue` (optional) - metadata to update (always merged by AI proxy)
- `PreprocessAction` - how to preprocess new input
- `ExecutionProvider` (optional) - hardware acceleration (e.g., CUDA)
- `ModelParams` - optional runtime parameters for the model
- `Schema` (optional) - schema namespace

## Source Code Example

<details>
  <summary>Click to expand source code</summary>

```go
package main

import (
  "context"
  "fmt"
  "log"
  "time"

  "google.golang.org/grpc"
  "google.golang.org/grpc/credentials/insecure"

  aiquery "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/query"
  aisvc "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/services/ai_service"
  keyval "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/keyval"
  metadata "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/metadata"
  predicates "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/predicates"
  preprocess "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/preprocess"
)

const AIAddr = "127.0.0.1:1370"

func stringPtr(value string) *string { return &value }

func main() {
  ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
  defer cancel()

  conn, err := grpc.DialContext(ctx, AIAddr,
      grpc.WithTransportCredentials(insecure.NewCredentials()),
      grpc.WithBlock(),
  )
  if err != nil {
      log.Fatalf("Failed to connect: %v", err)
  }
  defer conn.Close()

  client := aisvc.NewAIServiceClient(conn)

  condition := &predicates.PredicateCondition{
      Kind: &predicates.PredicateCondition_Value{
          Value: &predicates.Predicate{
              Kind: &predicates.Predicate_Equals{
                  Equals: &predicates.Equals{
                      Key: "filename",
                      Value: &metadata.MetadataValue{
                          Value: &metadata.MetadataValue_RawString{RawString: "photo.jpg"},
                      },
                  },
              },
          },
      },
  }

  newValue := &keyval.StoreValue{
      Value: map[string]*metadata.MetadataValue{
          "tags": {Value: &metadata.MetadataValue_RawString{RawString: "cat,outdoors"}},
      },
  }

  resp, err := client.Upsert(ctx, &aiquery.Upsert{
      Store:             "images",
      Schema:            stringPtr("media"),
      Condition:         condition,
      NewInput:          nil, // Optional: new image/text to re-embed
      NewValue:          newValue,
      PreprocessAction:  preprocess.PreprocessAction_NoPreprocessing,
      ExecutionProvider: nil,
      ModelParams:       map[string]string{},
  })
  if err != nil {
      log.Fatalf("Upsert failed: %v", err)
  }
  fmt.Printf("Updated: %d\n", resp.Upsert.Updated)
}
```

</details>

## Behavior

- AI proxy always merges metadata (preserves AI-generated fields)
- Errors if the predicate matches 0 or multiple entries
- Re-embeds input if `NewInput` is provided
- Returns upsert counts (inserted: 0, updated: 1 on success)
