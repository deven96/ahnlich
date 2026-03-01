---
title: Get Store
---

# Get Store

## Description

The `GetStore` request retrieves detailed information about a specific AI store by name, including the configured models and optional underlying DB store information.

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

    aisvc   "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/services/ai_service"
    aiquery "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/query"
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

func (c *ExampleAIClient) exampleGetStore() error {
    resp, err := c.client.GetStore(c.ctx, &aiquery.GetStore{
        Store: "ai_store",
    })
    if err != nil {
        return err
    }

    fmt.Printf("Store name: %s\n", resp.Name)
    fmt.Printf("Query model: %v\n", resp.QueryModel)
    fmt.Printf("Index model: %v\n", resp.IndexModel)
    fmt.Printf("Embedding size: %d\n", resp.EmbeddingSize)
    fmt.Printf("Dimension: %d\n", resp.Dimension)
    fmt.Printf("Predicate indices: %v\n", resp.PredicateIndices)

    if resp.DbInfo != nil {
        fmt.Printf("DB store size: %d bytes\n", resp.DbInfo.SizeInBytes)
    }

    return nil
}

func main() {
    ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
    defer cancel()

    client, err := NewAIClient(ctx)
    if err != nil {
        log.Fatalf("Failed to create AI client: %v", err)
    }
    defer client.Close()

    if err := client.exampleGetStore(); err != nil {
        log.Fatalf("GetStore failed: %v", err)
    }
}
```

</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `Store` | `string` | Yes | The name of the AI store to retrieve |

## Response: AIStoreInfo

| Field | Type | Description |
|-------|------|-------------|
| `Name` | `string` | Store name |
| `QueryModel` | `AIModel` | AI model used for query embeddings |
| `IndexModel` | `AIModel` | AI model used for index embeddings |
| `EmbeddingSize` | `uint64` | Number of stored embeddings |
| `Dimension` | `uint32` | Vector dimension (determined by model) |
| `PredicateIndices` | `[]string` | List of indexed predicate keys |
| `DbInfo` | `*StoreInfo` | Underlying DB store info (optional) |

## Notes

- Returns an error if the store does not exist
- The `DbInfo` field is present when the AI proxy is connected to a DB instance
- Use `ListStores` to get information about all AI stores
