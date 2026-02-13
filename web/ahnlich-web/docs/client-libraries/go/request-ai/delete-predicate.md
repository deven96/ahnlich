---
title: Delete Predicate
---

# Delete Predicate

## Description

The `DeletePredicate` request removes entries from an AI store that match a given predicate condition. This is a passthrough operation to the underlying DB service. Instead of deleting by vector key, this operation lets you **filter deletions based on metadata values** (e.g., labels, tags, or custom attributes).

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


    aisvc      "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/services/ai_service"
    aiquery    "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/query"
    metadata   "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/metadata"
    predicates "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/predicates"
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


// -------------------- Delete Predicate --------------------
func (c *ExampleAIClient) exampleDeletePredicate() error {
    condition := &predicates.PredicateCondition{
        Kind: &predicates.PredicateCondition_Value{
            Value: &predicates.Predicate{
                Kind: &predicates.Predicate_Equals{
                    Equals: &predicates.Equals{
                        Key: "category",
                        Value: &metadata.MetadataValue{
                            Value: &metadata.MetadataValue_RawString{
                                RawString: "archived",
                            },
                        },
                    },
                },
            },
        },
    }

    _, err := c.client.DelPred(c.ctx, &aiquery.DelPred{
        Store:     "my_ai_store",
        Condition: condition,
    })
    if err != nil {
        return err
    }
    fmt.Println("Deleted entries matching predicate from AI store: my_ai_store")
    return nil
}


// -------------------- Main --------------------
func main() {
    ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
    defer cancel()


    client, err := NewAIClient(ctx)
    if err != nil {
        log.Fatalf("Failed to create AI client: %v", err)
    }
    defer client.Close()


    if err := client.exampleDeletePredicate(); err != nil {
        log.Fatalf("DeletePredicate failed: %v", err)
    }
}
```

</details>

## What the code does

- Builds a predicate condition to match all entries where category == "archived".

- Sends a DelPred request against the AI store "my_ai_store".

- This is a passthrough to the underlying DB service, removing all vectors/entries that satisfy the predicate.
