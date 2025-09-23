---
title: Create Non-Linear algorithm Index
---

# Create Non-Linear algorithm Index

## Description

The `Create Non Linear Algorithm Index` request allows the AI service to build specialized non-linear search indices (e.g., KD-Tree) on top of vector embeddings that have already been stored in an AI-managed store.

Non-linear indices are essential when scaling similarity search, as they provide faster and more efficient retrieval of high-dimensional vectors compared to brute-force search.

When using this API, you explicitly tell the AI proxy to create an auxiliary search structure that accelerates queries. The AI system first embeds the raw input (e.g., text, image), stores the vectors in the underlying DB, and then attaches the non-linear algorithm index for faster retrieval.

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
  nonlinear "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/algorithm/nonlinear"
)


const AIAddr = "127.0.0.1:1370"


// ExampleAIClient wraps the connection + AIService client
type ExampleAIClient struct {
  conn   *grpc.ClientConn
  client aisvc.AIServiceClient
  ctx    context.Context
}


// NewAIClient connects to the AI server
func NewAIClient(ctx context.Context) (*ExampleAIClient, error) {
  conn, err := grpc.DialContext(ctx, AIAddr,
      grpc.WithTransportCredentials(insecure.NewCredentials()),
      grpc.WithBlock(),
  )
  if err != nil {
      return nil, fmt.Errorf("failed to dial AI server %q: %w", AIAddr, err)
  }
  client := aisvc.NewAIServiceClient(conn)
  return &ExampleAIClient{conn: conn, client: client, ctx: ctx}, nil
}


func (c *ExampleAIClient) Close() error {
  return c.conn.Close()
}


// ---- CreateNonLinearAlgorithmIndex ----
func (c *ExampleAIClient) exampleCreateNonLinearIndexAI() error {
  _, err := c.client.CreateNonLinearAlgorithmIndex(c.ctx, &aiquery.CreateNonLinearAlgorithmIndex{
      Store:            "ai_store",
      NonLinearIndices: []nonlinear.NonLinearAlgorithm{nonlinear.NonLinearAlgorithm_KDTree},
  })
  if err != nil {
      return err
  }
  fmt.Println(" Successfully created NonLinearAlgorithm index: KDTree on store ai_store")
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


  if err := client.exampleCreateNonLinearIndexAI(); err != nil {
      log.Fatalf(" CreateNonLinearAlgorithmIndex failed: %v", err)
  }
}

```

</details>

## Behavior

If the store already has embeddings, the index is constructed on them. Future inserts will also be indexed automatically.
