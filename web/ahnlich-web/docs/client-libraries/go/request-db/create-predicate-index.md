---
title: Create Predicate Index
---

# Create Predicate Index

## Description

`CreatePredicateIndex` allows you to create indexes on metadata fields inside a store. Predicate indexes speed up queries like `GetByPredicate`, especially when you frequently filter by the same metadata key.

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


    dbsvc   "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/services/db_service"
    dbquery "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/db/query"
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


// -------------------- Create Predicate Index --------------------
func (c *ExampleDBClient) exampleCreatePredicateIndex() error {
    _, err := c.client.CreatePredIndex(c.ctx, &dbquery.CreatePredIndex{
        Store:      "my_store",
        Predicates: []string{"label"},
    })
    if err != nil {
        return err
    }
    fmt.Println("Predicate index created for store: my_store")
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


    if err := client.exampleCreatePredicateIndex(); err != nil {
        log.Fatalf("CreatePredicateIndex failed: %v", err)
    }
}

```

</details>

## What the code does

- Creates a **predicate index** on the `label` metadata key within the store `my_store`.

- Once created, queries such as `GetByPredicate` that check `WHERE label = "X"` will run much faster.

- Returns any error encountered when creating the index (e.g., if it already exists).
