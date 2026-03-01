---
title: Create Non-Linear Algorithm Index
---

# Create Non-Linear Algorithm Index

## Description

The `CreateNonLinearAlgorithmIndex` request allows you to build specialized indices for **non-linear similarity search algorithms**. Unlike linear approaches (such as cosine or Euclidean), non-linear algorithms (like KD-Tree and HNSW) are optimized for faster and more scalable vector searches, especially as the dataset grows in size.

Each index type is specified using a `NonLinearIndex` message with either a `KDTreeConfig` or `HNSWConfig`.

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


func (c *ExampleDBClient) exampleCreateNonLinearAlgoIndex() error {
    // Create a KDTree index
    _, err := c.client.CreateNonLinearAlgorithmIndex(c.ctx, &dbquery.CreateNonLinearAlgorithmIndex{
        Store: "my_store",
        NonLinearIndices: []*nonlinear.NonLinearIndex{
            {Index: &nonlinear.NonLinearIndex_Kdtree{Kdtree: &nonlinear.KDTreeConfig{}}},
        },
    })
    if err != nil { return err }

    // Or create an HNSW index (with optional config)
    _, err = c.client.CreateNonLinearAlgorithmIndex(c.ctx, &dbquery.CreateNonLinearAlgorithmIndex{
        Store: "my_store",
        NonLinearIndices: []*nonlinear.NonLinearIndex{
            {Index: &nonlinear.NonLinearIndex_Hnsw{Hnsw: &nonlinear.HNSWConfig{}}},
        },
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


    if err := client.exampleCreateNonLinearAlgoIndex(); err != nil {
        log.Fatalf("CreateNonLinearAlgoIndex failed: %v", err)
    }
    fmt.Println("Created Non-Linear Algorithm Index for 'my_store'")
}
```

</details>
