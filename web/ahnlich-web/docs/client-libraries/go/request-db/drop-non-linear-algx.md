---
title: Drop Non-Linear Algorithm Index
---

# Drop Non-Linear Algorithm Index

## Description

The `DropNonLinearAlgorithmIndex` request removes previously created non-linear indices (e.g., KD-Tree) from a store. This is useful when you no longer need a specific algorithm index or want to reclaim resources used for maintaining it.

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
    nonlinear  "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/algorithm/nonlinear"
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


func (c *ExampleDBClient) exampleDropNonLinearAlgoIndex() error {
    _, err := c.client.DropNonLinearAlgorithmIndex(c.ctx, &dbquery.DropNonLinearAlgorithmIndex{
        Store:            "my_store",
        NonLinearIndices: []nonlinear.NonLinearAlgorithm{nonlinear.NonLinearAlgorithm_KDTree},
        ErrorIfNotExists: true,
    })
    return err
}


func main() {
    ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
    defer cancel()


    client, err := NewDBClient(ctx)
    if err != nil {
        log.Fatalf("Failed to create DB client: %v", err)
    }
    defer client.Close()


    if err := client.exampleDropNonLinearAlgoIndex(); err != nil {
        log.Fatalf("DropNonLinearAlgoIndex failed: %v", err)
    }
    fmt.Println("Dropped Non-Linear Algorithm Index for 'my_stores'")
}
```

</details>
