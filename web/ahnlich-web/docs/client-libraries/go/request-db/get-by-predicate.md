---
title: Get by Predicate
---

# Get by Predicate

## Description

`GetByPredicate` returns entries whose **metadata** matches a specified condition. Unlike similarity search, this is a **metadata-only** lookup (e.g., “all items where `label == "A"`”). Use it to filter or audit data based on tags, categories, or other metadata fields you maintain alongside vectors.

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


    dbsvc      "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/services/db_service"
    dbquery    "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/db/query"
    metadata   "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/metadata"
    predicates "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/predicates"
)


const ServerAddr = "127.0.0.1:1369"


// ExampleDBClient holds the gRPC connection, client, and context.
type ExampleDBClient struct {
    conn   *grpc.ClientConn
    client dbsvc.DBServiceClient
    ctx    context.Context
}


// NewDBClient connects to the Ahnlich DB server.
func NewDBClient(ctx context.Context) (*ExampleDBClient, error) {
    conn, err := grpc.DialContext(ctx, ServerAddr, grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithBlock())
    if err != nil {
        return nil, fmt.Errorf("failed to dial DB server %q: %w", ServerAddr, err)
    }
    client := dbsvc.NewDBServiceClient(conn)
    return &ExampleDBClient{conn: conn, client: client, ctx: ctx}, nil
}


// Close closes the gRPC connection.
func (c *ExampleDBClient) Close() error {
    return c.conn.Close()
}


// -------------------- Get By Predicate --------------------
func (c *ExampleDBClient) exampleGetByPredicate() error {
    // Build a PredicateCondition for "label == A"
    cond := &predicates.PredicateCondition{
        Kind: &predicates.PredicateCondition_Value{
            Value: &predicates.Predicate{
                Kind: &predicates.Predicate_Equals{
                    Equals: &predicates.Equals{
                        Key: "label",
                        Value: &metadata.MetadataValue{
                            Value: &metadata.MetadataValue_RawString{RawString: "A"},
                        },
                    },
                },
            },
        },
    }


    // Call GetPred with the condition
    resp, err := c.client.GetPred(c.ctx, &dbquery.GetPred{
        Store:     "my_store",
        Condition: cond,
    })
    if err != nil {
        return err
    }


    fmt.Println("GetByPredicate Results:", resp.Entries)
    return nil
}


// -------------------- Main --------------------
func main() {
    ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
    defer cancel()


    client, err := NewDBClient(ctx)
    if err != nil {
        log.Fatalf("Failed to create DB client: %v", err)
    }
    defer client.Close()


    if err := client.exampleGetByPredicate(); err != nil {
        log.Fatalf("GetByPredicate failed: %v", err)
    }
}

```

</details>

## What the code does

- Builds a predicate condition equivalent to: **WHERE** `label` **==** `"A"`.

- Sends the request to store `my_store` and prints matching entries (`resp.Entries`).

- Returns any RPC error to the caller.
