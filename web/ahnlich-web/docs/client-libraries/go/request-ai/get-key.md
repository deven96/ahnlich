---
title: Get Key
---

# Get Key

## Description

The `GetKey` request retrieves specific entries from an AI store by their keys. Unlike similarity search, this is a **direct lookup** operation that returns exact matches for the provided keys along with their metadata and stored values.

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


  aiquery  "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/query"
  aisvc    "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/services/ai_service"
  keyval   "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/keyval"
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


// ---- GetKey Example ----
func (c *ExampleAIClient) exampleGetKey() error {
  resp, err := c.client.GetKey(c.ctx, &aiquery.GetKey{
      Store: "ai_store01",
      Keys: []*keyval.StoreInput{
          {Value: &keyval.StoreInput_RawString{RawString: "Adidas Yeezy"}},
          {Value: &keyval.StoreInput_RawString{RawString: "Nike Air Jordans"}},
      },
  })
  if err != nil {
      return err
  }

  fmt.Println("GetKey Results:")
  for _, entry := range resp.Entries {
      fmt.Printf("  Key: %v\n", entry.Key)
      fmt.Printf("  Value: %v\n", entry.Value)
  }
  return nil
}


func main() {
  ctx, cancel := context.WithTimeout(context.Background(), 15*time.Second)
  defer cancel()


  client, err := NewAIClient(ctx)
  if err != nil {
      log.Fatalf("Failed to create AI client: %v", err)
  }
  defer client.Close()


  if err := client.exampleGetKey(); err != nil {
      log.Fatalf("GetKey failed: %v", err)
  }
}
```

</details>

## Behavior

- **Store** – The target AI store that must already exist.

- **Keys** – List of `StoreInput` keys to retrieve. Each key is the original input (text/image) that was used when storing the data.

- The response contains entries with the stored data and metadata for each found key.

- If a key doesn't exist in the store, it won't appear in the results (no error is returned for missing keys).

- This is useful for retrieving known items by their exact key, such as looking up specific documents or images you previously stored.
