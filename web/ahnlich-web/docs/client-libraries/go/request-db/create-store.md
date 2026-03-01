---
title: Create Store
---

# Create Store

## Description

`CreateStore` creates a new vector store on the Ahnlich DB server. A store is the fundamental unit of organization for embeddings and their associated metadata. Each store is defined by its **name** and its **dimension**, which specifies the length of vectors that can be inserted.

## Behavior

- A store must have a **unique name** within the server instance. If you try to create a store with an existing name and `ErrorIfExists` is true, the server will reject it.

- The **dimension parameter** is mandatory and must match the size of vectors you plan to insert.

- **CreatePredicates** - Optional list of metadata field names to enable predicate-based filtering. Leave empty if you don't need metadata filtering.

- **NonLinearIndices** - Optional list of non-linear algorithms for faster approximate search. Leave empty to use only linear search.

- **ErrorIfExists** - Controls behavior when store already exists. Set to `true` to get an error, `false` to silently skip creation.

- Once created, a store can be queried, listed, and populated with embeddings.

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


    dbsvc     "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/services/db_service"
    dbquery   "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/db/query"
    nonlinear "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/algorithm/nonlinear"
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


func (c *ExampleDBClient) Close() error { return c.conn.Close() }


// CreateStore example
func (c *ExampleDBClient) exampleCreateStore(store string, dimension int32) error {
    _, err := c.client.CreateStore(c.ctx, &dbquery.CreateStore{
        Store:             store,
        Dimension:         uint32(dimension),
        CreatePredicates:  []string{}, // Optional: list of metadata fields to index for filtering
        NonLinearIndices:  []*nonlinear.NonLinearIndex{},  // Optional: non-linear algorithms (e.g., KDTree, HNSW) for faster search
        ErrorIfExists:     true,        // Return error if store already exists
    })
    if err != nil {
        return err
    }
    fmt.Printf("Created store: %s (dimension: %d)\n", store, dimension)
    return nil
}


func main() {
    ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
    defer cancel()


    client, err := NewDBClient(ctx)
    if err != nil {
        log.Fatalf("Failed to create DB client: %v", err)
    }
    defer client.Close()


    storeName := "my_stores"
    if err := client.exampleCreateStore(storeName, 4); err != nil {
        log.Fatalf("CreateStore failed: %v", err)
    }
}
```

</details>

This method requests the creation of a store named `"my_store"` with a vector dimensionality of 4. The response is ignored in this example, but the absence of an error indicates success.
