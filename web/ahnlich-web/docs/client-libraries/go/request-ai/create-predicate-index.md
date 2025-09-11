---
title: Create Predicate Index
---

# Create Predicate Index

## Description

The **CreatePredicateIndex** request enhances retrieval efficiency by building indexes on specified metadata fields within an AI-managed store. By indexing metadata keys, subsequent GetByPredicate requests can be executed more efficiently, especially on large datasets.

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


// ---- CreatePredIndex  ----
func (c *ExampleAIClient) exampleCreatePredIndexAI() error {
  _, err := c.client.CreatePredIndex(c.ctx, &aiquery.CreatePredIndex{
      Store:      "ai_store",
      Predicates: []string{"f"},
  })
  if err != nil {
      return err
  }
  fmt.Println(`Predicate index created successfully with logic: Predicates: []string{"f"},`)
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


  if err := client.exampleCreatePredIndexAI(); err != nil {
      log.Fatalf(" CreatePredIndex failed: %v", err)
  }
}
```

</details>

## Behavior

- After this index is created, queries filtering on the `"f"` field (e.g., via `GetByPredicate`) are **faster** and more scalable.

- Multiple predicate fields can be indexed by including them in the `Predicates` array.

- Attempting to recreate an existing index may return an error depending on configuration.

- Predicate indexes complement embedding-based retrieval, enabling **hybrid query patterns** that combine metadata filtering with similarity search.
