---
title: List Stores
---

# List Stores

## Description

`ListStores` provides a **catalog of AI-managed stores** currently registered in the Ahnlich AI proxy. Each store represents a logical grouping where raw inputs (text, images, etc.) are converted into embeddings using the configured models and then indexed for retrieval.

This request allows developers to:

- Discover all existing stores available for AI-based operations.

- Verify that newly created stores are correctly registered and visible.

- Audit what data partitions are accessible through the AI proxy at a given time.

It is especially useful in **multi-tenant scenarios**, where different applications or teams may own separate stores, and clients need visibility into what stores exist before performing further operations.

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


// ExampleAIClient holds the gRPC connection and AI client.
type ExampleAIClient struct {
    conn   *grpc.ClientConn
    client aisvc.AIServiceClient
    ctx    context.Context
}


// NewAIClient connects to the AI server and returns a client.
func NewAIClient(ctx context.Context) (*ExampleAIClient, error) {
    conn, err := grpc.DialContext(ctx, AIAddr, grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithBlock())
    if err != nil {
        return nil, fmt.Errorf("failed to dial AI server %q: %w", AIAddr, err)
    }
    client := aisvc.NewAIServiceClient(conn)
    return &ExampleAIClient{conn: conn, client: client, ctx: ctx}, nil
}


// Close closes the gRPC connection.
func (c *ExampleAIClient) Close() error {
    return c.conn.Close()
}


// ---- ListStores Example ----
// List all stores available on the AI server.
func (c *ExampleAIClient) exampleListStoresAI() error {
    resp, err := c.client.ListStores(c.ctx, &aiquery.ListStores{})
    if err != nil {
        return err
    }
    fmt.Println(" AI Stores:", resp.Stores)
    return nil
}


// ---- MAIN ----
func main() {
    ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
    defer cancel()


    client, err := NewAIClient(ctx)
    if err != nil {
        log.Fatalf("Failed to create AI client: %v", err)
    }
    defer client.Close()


    if err := client.exampleListStoresAI(); err != nil {
        log.Fatalf("ListStores failed: %v", err)
    }
}

```

</details>

## Behavior

- The response represents the set of AI-managed stores the proxy reports at the time of the request.

- An empty **resp.Stores** means the AI proxy currently reports no configured stores.

- The server’s list is authoritative for that moment in time; contents may change between calls.

- A non-nil error indicates the request failed and no valid store list was returned.
