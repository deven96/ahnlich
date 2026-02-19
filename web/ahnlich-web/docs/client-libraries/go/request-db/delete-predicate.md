---
title: Delete Predicate
---

# Delete Predicate

## Description

The `DeletePredicate` request removes entries from a store that match a given predicate condition. Instead of deleting by vector key, this operation lets you **filter deletions based on metadata values** (e.g., labels, tags, or custom attributes).

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


type ExampleDBClient struct {
    conn   *grpc.ClientConn
    client dbsvc.DBServiceClient
    ctx    context.Context
}


func NewDBClient(ctx context.Context) (*ExampleDBClient, error) {
    conn, err := grpc.DialContext(ctx, ServerAddr, grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithBlock())
    if err != nil {
        return nil, fmt.Errorf("failed to dial DB server %q: %w", ServerAddr, err)
    }
    client := dbsvc.NewDBServiceClient(conn)
    return &ExampleDBClient{conn: conn, client: client, ctx: ctx}, nil
}


func (c *ExampleDBClient) Close() error {
    return c.conn.Close()
}


// -------------------- Delete Predicate --------------------
func (c *ExampleDBClient) exampleDeletePredicate() error {
    condition := &predicates.PredicateCondition{
        Kind: &predicates.PredicateCondition_Value{
            Value: &predicates.Predicate{
                Kind: &predicates.Predicate_Equals{
                    Equals: &predicates.Equals{
                        Key: "label",
                        Value: &metadata.MetadataValue{
                            Value: &metadata.MetadataValue_RawString{
                                RawString: "A",
                            },
                        },
                    },
                },
            },
        },
    }

    _, err := c.client.DelPred(c.ctx, &dbquery.DelPred{
        Store:     "my_store",
        Condition: condition,
    })
    if err != nil {
        return err
    }
    fmt.Println("Deleted entries matching predicate from store: my_store")
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


    if err := client.exampleDeletePredicate(); err != nil {
        log.Fatalf("DeletePredicate failed: %v", err)
    }
}
```

</details>

## What the code does

- Builds a predicate condition to match all entries where label == "A".

- Sends a DelPred request against the store "my_store".

- Removes all vectors/entries that satisfy the predicate.
