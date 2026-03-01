---
title: List Connected Clients
---

# List Connected Clients

## Description

The `ListConnectedClients` request allows you to query the AI server for all currently connected clients. This is particularly useful for **monitoring, debugging,** and ensuring that multiple services or developers interacting with the same Ahnlich AI instance are tracked properly.

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

func (c *ExampleAIClient) exampleListConnectedClients() error {
    resp, err := c.client.ListClients(c.ctx, &aiquery.ListClients{})
    if err != nil {
        return err
    }
    fmt.Println("Connected Clients:", resp.Clients)
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

    if err := client.exampleListConnectedClients(); err != nil {
        log.Fatalf("ListConnectedClients failed: %v", err)
    }
}
```

</details>

This method requests the list of all currently connected clients to the Ahnlich AI server. The response contains client information including addresses.
