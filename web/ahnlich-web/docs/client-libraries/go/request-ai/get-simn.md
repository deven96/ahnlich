---
title: GetSimN
---

# GetSimN

## Description

The `GetSimN` request-ai performs a **nearest-neighbor search** using a **raw query input**. The AI proxy embeds the raw query with the store’s configured **QueryModel**, sends the similarity search to the DB, and returns the top-N matches.

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


  aiquery    "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/query"
  aisvc      "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/services/ai_service"
  keyval     "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/keyval"
  metadata   "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/metadata"
  algorithms "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/algorithm/algorithms"
  preprocess "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/preprocess"
  predicates "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/predicates"
)


const AIAddr = "127.0.0.1:1370"


type ExampleAIClient struct {
  conn   *grpc.ClientConn
  client aisvc.AIServiceClient
  ctx    context.Context
}


func NewAIClient(ctx context.Context) (*ExampleAIClient, error) {
  conn, err := grpc.DialContext(ctx, AIAddr, grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithBlock())
  if err != nil {
      return nil, fmt.Errorf("failed to dial AI server %q: %w", AIAddr, err)
  }
  client := aisvc.NewAIServiceClient(conn)
  return &ExampleAIClient{conn: conn, client: client, ctx: ctx}, nil
}


func (c *ExampleAIClient) Close() error {
  return c.conn.Close()
}


// Helper to unwrap Key (StoreInput)
func unwrapKey(k *keyval.StoreInput) string {
  if k == nil {
      return "<nil>"
  }
  switch v := k.Value.(type) {
  case *keyval.StoreInput_RawString:
      return v.RawString
  default:
      return fmt.Sprintf("%v", v)
  }
}


// Helper to unwrap Value (StoreValue)
func unwrapValue(v *keyval.StoreValue) map[string]string {
  result := make(map[string]string)
  if v == nil {
      return result
  }
  for k, val := range v.Value {
      switch mv := val.Value.(type) {
      case *metadata.MetadataValue_RawString:
          result[k] = mv.RawString
      default:
          result[k] = fmt.Sprintf("%v", mv)
      }
  }
  return result
}


// ---- GetSimN  ----
func (c *ExampleAIClient) exampleGetSimNAI() error {
  resp, err := c.client.GetSimN(c.ctx, &aiquery.GetSimN{
      Store:            "ai_store01", // must already exist and have data
      SearchInput:      &keyval.StoreInput{Value: &keyval.StoreInput_RawString{RawString: "X"}},
      Condition:        nil, // Optional: filter results using predicates
      ClosestN:         3,
      Algorithm:        algorithms.Algorithm_CosineSimilarity,
      PreprocessAction: preprocess.PreprocessAction_ModelPreprocessing, // Apply model's preprocessing
      ExecutionProvider: nil, // Optional: e.g., ExecutionProvider_CUDA for GPU acceleration
  })
  if err != nil {
      return err
  }


  fmt.Println(" AI GetSimN Response:")
  for i, entry := range resp.Entries {
      fmt.Printf("  #%d Key=%s  Value=%v\n", i+1, unwrapKey(entry.Key), unwrapValue(entry.Value))
  }
  return nil
}


func main() {
  ctx, cancel := context.WithTimeout(context.Background(), 15*time.Second)
  defer cancel()


  client, err := NewAIClient(ctx)
  if err != nil {
      log.Fatalf(" Failed to create AI client: %v", err)
  }
  defer client.Close()


  if err := client.exampleGetSimNAI(); err != nil {
      log.Fatalf(" GetSimN failed: %v", err)
  }
}
```

</details>

## Behavior

- **Store** – The target AI store that must already exist.

- **SearchInput** – Raw input (text or image) that gets embedded using the store's QueryModel.

- **Condition** – Optional predicate filter to restrict which vectors are considered. Set to `nil` to search all vectors. See [Predicates documentation](/components/predicates/predicates).

- **ClosestN** – Number of top matches to return.

- **Algorithm** – Similarity metric to use (`CosineSimilarity`, `EuclideanDistance`, `DotProductSimilarity`).

- **PreprocessAction** – Controls input preprocessing:
  - `ModelPreprocessing` – Apply model's built-in preprocessing (recommended for most cases)
  - `NoPreprocessing` – Skip preprocessing (use if you've already preprocessed the input)

- **ExecutionProvider** – Optional hardware acceleration (e.g., `CUDA`, `TensorRT`, `CoreML`). Set to `nil` to use default CPU execution.

- The AI proxy **embeds** the raw `SearchInput` with the store's **QueryModel** and forwards the search to the DB.

- The response contains up to `ClosestN` matching entries (`resp.Entries`) from the store (entries include stored vectors' associated data/metadata).

- Results depend on what has been previously ingested (e.g., via AI `Set` or direct DB ingestion).

- If no neighbors are found, the call succeeds with an empty result set; failures return a non-nil error.
