---
title: Drop Store
---

# Drop Store

## Description

The **DropStore** request deletes an entire AI-managed store, including its embeddings, metadata, and indexes. This is a destructive operation and should be used when a store is no longer required.

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
)


const AIAddr = "127.0.0.1:1370"


// ExampleAIClient wraps the connection + AIService client
type ExampleAIClient struct {
  conn   *grpc.ClientConn
  client aisvc.AIServiceClient
  ctx    context.Context
}


// NewAIClient creates and connects the AI client
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


// ---- DropStore  ----
func (c *ExampleAIClient) exampleDropStoreAI() error {
  _, err := c.client.DropStore(c.ctx, &aiquery.DropStore{
      Store:            "ai_store01",
      ErrorIfNotExists: true,
  })
  if err != nil {
      return err
  }
  fmt.Println(" Successfully dropped store: ai_store")
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


  if err := client.exampleDropStoreAI(); err != nil {
      log.Fatalf(" DropStore failed: %v", err)
  }
}

```

</details>

## Behavior

- Permanently deletes all embeddings, metadata, and indexes in the store.

- After execution, any queries targeting `"ai_store"` will fail until the store is recreated.

- Useful for reclaiming resources or resetting the environment.

- If `ErrorIfNotExists` is set to `false`, the operation becomes idempotent (no error on missing stores).
