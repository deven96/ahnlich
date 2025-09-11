---
title: Ping
---

# Ping

## Description

The `Ping` request verifies basic connectivity and reachability between your Go client and the **Ahnlich AI proxy**. It’s the simplest call you can make to confirm the AI service is available before sending heavier requests (for example, embedding or create-store operations).

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


// ---- Ping Example ----
func (c *ExampleAIClient) examplePingAI() error {
  resp, err := c.client.Ping(c.ctx, &aiquery.Ping{})
  if err != nil {
      return err
  }
  fmt.Println(" AI Ping:", resp)
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


  if err := client.examplePingAI(); err != nil {
      log.Fatalf("Ping failed: %v", err)
  }
}
```

</details>

## Behavior

A successful call indicates the client can reach and receive a response from the AI proxy.

A failure (non-nil err) typically indicates connectivity problems or that the AI proxy is not currently accepting requests.

The call should be fast and lightweight — suitable for frequent checks if needed, but treat it as an RPC (avoid unnecessarily tight polling loops).
