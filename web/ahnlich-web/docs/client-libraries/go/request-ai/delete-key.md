---
title: Delete Key
---

# Delete Key

## Description

The **DeleteKey** request removes specific entries from an AI-managed store using their **input keys**. Unlike dropping the store entirely, this operation targets only selected records.

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
  keyval "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/keyval"
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


// ---- DelKey  ----
func (c *ExampleAIClient) exampleDeleteKeyAI() error {
  _, err := c.client.DelKey(c.ctx, &aiquery.DelKey{
      Store: "ai_store01",
      Keys: []*keyval.StoreInput{
          {Value: &keyval.StoreInput_RawString{RawString: "X"}},
      },
  })
  if err != nil {
      return err
  }
  fmt.Println(" Successfully deleted key 'X' from store ai_store")
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


  if err := client.exampleDeleteKeyAI(); err != nil {
      log.Fatalf(" DeleteKey failed: %v", err)
  }
}
```

</details>

## Behavior

- Only the entries associated with the provided key(s) are removed.

- Supports deleting multiple keys at once by passing more StoreInput values.

- If a key does not exist, behavior depends on configuration—it may silently ignore or return an error.

- Useful for targeted cleanup without affecting the rest of the store.
