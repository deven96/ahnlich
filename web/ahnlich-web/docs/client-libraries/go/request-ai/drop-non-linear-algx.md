---
title: Drop Non-Linear Algorithm Index
---

# Drop Non-Linear Algorithm Index

## Description

The **Drop Non Linear Algorithm Index** request removes a previously created non-linear index from a store. This operation is important when you want to reclaim resources, update to a different algorithm, or revert to default brute-force scanning for search.

By dropping the index, the AI proxy no longer uses the KDTree (or any other specified algorithm) for accelerating queries. Instead, searches will fall back to direct vector comparisons.

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


// ---- DropNonLinearAlgorithmIndex standalone ----
func (c *ExampleAIClient) exampleDropNonLinearIndexAI() error {
  _, err := c.client.DropNonLinearAlgorithmIndex(c.ctx, &aiquery.DropNonLinearAlgorithmIndex{
      Store:            "ai_store",
      NonLinearIndices: []nonlinear.NonLinearAlgorithm{nonlinear.NonLinearAlgorithm_KDTree},
      ErrorIfNotExists: true,
  })
  if err != nil {
      return err
  }
  fmt.Println(" Successfully dropped NonLinearAlgorithm index: KDTree from store ai_store")
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


  if err := client.exampleDropNonLinearIndexAI(); err != nil {
      log.Fatalf(" DropNonLinearAlgorithmIndex failed: %v", err)
  }
}
```

</details>

## Behavior:

- **Target Store**: Operates on `"ai_store"`.

- **Drop Target**: Removes the **KDTree** index from the store.

- **Error Handling**: With `ErrorIfNotExists`: `true`, the request will fail if no such index exists.

- **Effect**: Queries against the store will **no longer benefit from KDTree acceleration**, reverting to full-scan or other available indices.
