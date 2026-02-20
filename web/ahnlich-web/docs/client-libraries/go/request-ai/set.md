---
title: Set
---

# Set

## Description

The `Set` request-ai ingests raw inputs (e.g., text) through the Ahnlich AI proxy. The proxy converts each raw input into an embedding using the store’s configured IndexModel and persists the result—together with any provided metadata—into the underlying Ahnlich DB store.

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
  preprocess "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/preprocess"
)


const AIAddr = "127.0.0.1:1370"


// ---- Standalone Set Example ----
func main() {
  ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
  defer cancel()


  // connect to AI server
  conn, err := grpc.DialContext(ctx, AIAddr,
      grpc.WithTransportCredentials(insecure.NewCredentials()),
      grpc.WithBlock(),
  )
  if err != nil {
      log.Fatalf(" Failed to connect to AI server: %v", err)
  }
  defer conn.Close()


  client := aisvc.NewAIServiceClient(conn)


  // prepare key/value input
  inputs := []*keyval.AiStoreEntry{
      {
          Key: &keyval.StoreInput{
              Value: &keyval.StoreInput_RawString{RawString: "X"},
          },
          Value: &keyval.StoreValue{
              Value: map[string]*metadata.MetadataValue{
                  "f": {Value: &metadata.MetadataValue_RawString{RawString: "v"}},
              },
          },
      },
  }


  // perform Set operation
  _, err = client.Set(ctx, &aiquery.Set{
      Store:             "ai_store", // must already exist
      Inputs:            inputs,
      PreprocessAction:  preprocess.PreprocessAction_NoPreprocessing,
      ExecutionProvider: nil, // Optional: e.g., ExecutionProvider_CUDA for GPU acceleration
  })
  if err != nil {
      log.Fatalf(" Set failed: %v", err)
  }


  fmt.Println(" Successfully inserted key/value into ai_store01")
}
```

</details>


## Behavior 
- On success, the AI proxy **embeds** the raw input using the store’s **IndexModel** and **stores** the embedding plus metadata in the DB-backed store.

- Multiple entries can be ingested by adding more items to `Inputs`.

- The target store (`"ai_store"`) must already exist and be configured with models in the AI proxy.

- If the request fails (e.g., store not found or server error), a non-nil error is returned.


