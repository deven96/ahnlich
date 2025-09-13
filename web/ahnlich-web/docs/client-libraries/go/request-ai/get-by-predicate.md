---
title: Get by Predicate
---

# Get by Predicate

## Description

The `GetByPredicate` request-ai retrieves entries from an AI-managed store by applying **metadata-based filtering**. Unlike similarity search (`GetSimN`), which operates on vector embeddings, predicate queries operate on **stored metadata attributes**. This enables developers to fetch records that satisfy specific metadata conditions, regardless of their embedding similarity.

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
    metadata "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/metadata"
    predicates "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/predicates"
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


// ---- GetByPredicate standalone ----
func (c *ExampleAIClient) exampleGetByPredicateAI() error {
    cond := &predicates.PredicateCondition{
        Kind: &predicates.PredicateCondition_Value{
            Value: &predicates.Predicate{
                Kind: &predicates.Predicate_Equals{
                    Equals: &predicates.Equals{
                        Key: "f",
                        Value: &metadata.MetadataValue{
                            Value: &metadata.MetadataValue_RawString{RawString: "v"},
                        },
                    },
                },
            },
        },
    }


    resp, err := c.client.GetPred(c.ctx, &aiquery.GetPred{
        Store:     "ai_store",
        Condition: cond,
    })
    if err != nil {
        return err
    }
    fmt.Println(" AI GetByPredicate Response:", resp.Entries)
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


    if err := client.exampleGetByPredicateAI(); err != nil {
        log.Fatalf(" GetByPredicate failed: %v", err)
    }
}

```

</details>

## What the code does

- Constructs a **predicate condition** that checks if the metadata field `"f"` equals the raw string `"v"`.

- Issues a `GetByPredicate` request against "`ai_store`".

- On success, prints the retrieved entries (`resp.Entries`); otherwise, returns the error.

## Behavior

- The AI proxy evaluates the condition against stored entries’ metadata.

- Only records with metadata key `"f"` equal to `"v"` are returned.

- Does not perform embedding similarity—this is purely metadata-based retrieval.

- If no matching entries exist, the response is valid but empty.

- Useful for filtering results in combination with embedding-based searches.
