---
title: GetSimN
---

# GetSimN

## Description

The `GetSimN` request-ai performs a **nearest-neighbor search** using a **raw query input**. The AI proxy embeds the raw query with the store’s configured **QueryModel**, sends the similarity search to the DB, and returns the top-N matches.

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
  metadata "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/metadata"
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


// Helper to unwrap Key (StoreInput)
func unwrapKey(k *keyval.StoreInput) string {
  if k == nil {
      return "<nil>"
  }
  switch v := k.Value.(type) {
  case *keyval.StoreInput_RawString:
      return v.RawString
  default:
      return fmt.Sprintf("%v", v)
  }
}


// Helper to unwrap Value (StoreValue)
func unwrapValue(v *keyval.StoreValue) map[string]string {
  result := make(map[string]string)
  if v == nil {
      return result
  }
  for k, val := range v.Value {
      switch mv := val.Value.(type) {
      case *metadata.MetadataValue_RawString:
          result[k] = mv.RawString
      default:
          result[k] = fmt.Sprintf("%v", mv)
      }
  }
  return result
}


// ---- GetSimN  ----
func (c *ExampleAIClient) exampleGetSimNAI() error {
  resp, err := c.client.GetSimN(c.ctx, &aiquery.GetSimN{
      Store:       "ai_store01", // must already exist and have data
      SearchInput: &keyval.StoreInput{Value: &keyval.StoreInput_RawString{RawString: "X"}},
      ClosestN:    3,
  })
  if err != nil {
      return err
  }


  fmt.Println(" AI GetSimN Response:")
  for i, entry := range resp.Entries {
      fmt.Printf("  #%d Key=%s  Value=%v\n", i+1, unwrapKey(entry.Key), unwrapValue(entry.Value))
  }
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


  if err := client.exampleGetSimNAI(); err != nil {
      log.Fatalf(" GetSimN failed: %v", err)
  }
}
```

</details>

## Behavior

- The AI proxy **embeds** the raw `SearchInput` with the store’s **QueryModel** and forwards the search to the DB.

- The response contains up to `ClosestN` matching entries (`resp.Entries`) from the store (entries include stored vectors’ associated data/metadata).

- Results depend on what has been previously ingested (e.g., via AI `Set` or direct DB ingestion).

- If no neighbors are found, the call succeeds with an empty result set; failures return a non-nil error.
