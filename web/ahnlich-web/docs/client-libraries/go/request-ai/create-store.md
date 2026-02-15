---
title: Create Store
---

# Create Store

## Description

`CreateStore` is a **provisioning request** used to define a new AI-backed store within the Ahnlich AI proxy. Unlike the raw DB `CreateStore`, this AI-specific request requires model selection for **both indexing** and **querying**. These models determine how raw inputs (e.g., text, images) are embedded and how queries against the store are interpreted.
Key concepts:

- **IndexModel**: the embedding model used to transform stored data into vector form.

- **QueryModel**: the embedding model used to transform incoming queries into vector form before comparison.

- **Store Name**: a unique identifier for the logical collection of embeddings.


This design allows developers to separate how data is stored vs. how queries are expressed. In most cases, the same model is chosen for both roles (as in the example), but they can differ if needed for domain-specific optimization.

## Use cases:

- Creating a dedicated store for text embeddings using a general-purpose model.

- Initializing multiple stores with different models for experimentation.

- Supporting hybrid workflows where one model indexes the data and another interprets queries.

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
    aimodel "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/models"
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


// ---- CreateStore Example ----
// Create a new store for AI operations.
func (c *ExampleAIClient) exampleCreateStoreAI() error {
    _, err := c.client.CreateStore(c.ctx, &aiquery.CreateStore{
        Store:            "ai_store",
        QueryModel:       aimodel.AIModel_ALL_MINI_LM_L6_V2,
        IndexModel:       aimodel.AIModel_ALL_MINI_LM_L6_V2,
        Predicates:       []string{},  // Optional: metadata fields to index for filtering
        NonLinearIndices: []int32{},   // Optional: non-linear algorithms for faster search
        ErrorIfExists:    true,         // Return error if store already exists
        StoreOriginal:    true,         // Store original input (needed for key deletion)
    })
    if err != nil {
        return err
    }
    fmt.Println(" AI Store created: ai_store01")
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


    if err := client.exampleCreateStoreAI(); err != nil {
        log.Fatalf("CreateStore failed: %v", err)
    }
}

```
</details>


Behavior
- On success, the AI proxy registers a new AI-backed store configured with the provided `IndexModel` and `QueryModel` (the models determine how raw inputs are converted to embeddings for indexing and querying).

- The `QueryModel` and `IndexModel` fields use the `aimodel` enum values supplied in the request; the proxy interprets those enums according to its supported model set. 

- Errors indicate the operation did not complete (for example, validation failure, naming conflict, or server-side error). The exact failure modes and error payloads are determined by the AI proxy implementation.
