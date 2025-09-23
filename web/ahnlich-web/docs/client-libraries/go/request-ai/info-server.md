---
title: Info Server
---

# Info Server

## Description

`InfoServer` is a lightweight **introspection request** that allows a client to query the **Ahnlich AI proxy** for its current status and metadata. This request does not modify any state; rather, it provides insights into the AI proxy’s operational environment.
This can include details such as:

- Server identification and build information.

- Current runtime configuration (as exposed by the server).

- Status checks to confirm the AI service is alive and responsive.

Developers typically use this request in monitoring or debugging workflows to validate that the AI proxy is online and correctly configured before sending heavier workloads (like store creation or query execution).

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


// ---- InfoServer Example ----
// Retrieve server information such as address, version, limits.
func (c *ExampleAIClient) exampleInfoServerAI() error {
  resp, err := c.client.InfoServer(c.ctx, &aiquery.InfoServer{})
  if err != nil {
      return err
  }
  fmt.Println(" AI InfoServer:", resp)
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


  if err := client.exampleInfoServerAI(); err != nil {
      log.Fatalf("InfoServer failed: %v", err)
  }
}
```

</details>

## Behavior

- The call is **read-only** and returns whatever server-level metadata the AI proxy exposes (identifiers, status/configuration details as provided by the service).

- A successful call yields a populated `resp`; the exact fields depend on the AI proxy’s implementation.

- A non-nil error indicates the RPC failed (network, server-side error, or other RPC-level failure).

- The call is intended to be lightweight and non-destructive.
