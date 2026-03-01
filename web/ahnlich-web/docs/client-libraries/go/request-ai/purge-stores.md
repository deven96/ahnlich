---
title: Purge Stores
---

# Purge Stores

## Description

Deletes **all vector stores** managed by the AI server, including all embeddings and associated metadata. This is a destructive operation that resets the AI service state, typically used during testing, cleanup, or when starting fresh with new datasets.

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

const ServerAddr = "127.0.0.1:1370"

type ExampleAIClient struct {
    conn   *grpc.ClientConn
    client aisvc.AIServiceClient
    ctx    context.Context
}

func NewAIClient(ctx context.Context) (*ExampleAIClient, error) {
    conn, err := grpc.DialContext(ctx, ServerAddr, grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithBlock())
    if err != nil {
        return nil, fmt.Errorf("failed to dial AI server %q: %w", ServerAddr, err)
    }
    client := aisvc.NewAIServiceClient(conn)
    return &ExampleAIClient{conn: conn, client: client, ctx: ctx}, nil
}

func (c *ExampleAIClient) Close() error {
    return c.conn.Close()
}

func (c *ExampleAIClient) examplePurgeStores() error {
    resp, err := c.client.PurgeStores(c.ctx, &aiquery.PurgeStores{})
    if err != nil {
        return err
    }
    fmt.Printf("Purged stores. Deleted count: %d\n", resp.DeletedCount)
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

    if err := client.examplePurgeStores(); err != nil {
        log.Fatalf("PurgeStores failed: %v", err)
    }
}
```

</details>

:::warning
This operation is **irreversible**. All stores and their data will be permanently deleted. Use with caution in production environments.
:::
