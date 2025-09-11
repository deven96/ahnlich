---
title: Drop Predicate Index
---

# Drop Predicate Index

## Description

The **DropPredicateIndex** request removes an existing predicate index from a given AI-managed store. Predicate indexes are used to accelerate metadata-based queries (`GetByPredicate`), and dropping them reverts queries to a slower scan-based evaluation.

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


// ---- DropPredIndex standalone ----
func (c *ExampleAIClient) exampleDropPredicateIndexAI() error {
  _, err := c.client.DropPredIndex(c.ctx, &aiquery.DropPredIndex{
      Store:            "ai_store",
      Predicates:       []string{"f"},
      ErrorIfNotExists: true,
  })
  if err != nil {
      return err
  }
  fmt.Println(" Successfully dropped predicate index for Predicates: [\"f\"] in store ai_store")
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


  if err := client.exampleDropPredicateIndexAI(); err != nil {
      log.Fatalf(" DropPredIndex failed: %v", err)
  }
}
```

</details>

## Behavior

- After removal, queries filtering on `"f"` remain valid but may become slower due to lack of indexing.

- Safe for cleaning up unused indexes.

- If `ErrorIfNotExists` is set to `false`, the operation is idempotent (no error even if index is missing).

- Recommended when predicate-based queries on `"f"` are no longer needed.
