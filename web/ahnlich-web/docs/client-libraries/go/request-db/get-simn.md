---
title: GetSimN
---

# GetSimN

## Description

The `GetSimN` request performs a **similarity search** against a specific store. It retrieves the top-N closest vectors to a given input vector. This operation is essential for applications that depend on **nearest neighbor lookups** such as recommendation systems, semantic search, and clustering.

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
    keyval  "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/keyval"
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
    conn, err := grpc.DialContext(
        ctx,
        ServerAddr,
        grpc.WithTransportCredentials(insecure.NewCredentials()),
        grpc.WithBlock(),
    )
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


// -------------------- GetSimN --------------------
func (c *ExampleDBClient) exampleGetSimN() error {
    resp, err := c.client.GetSimN(c.ctx, &dbquery.GetSimN{
        Store:       "my_store",
        SearchInput: &keyval.StoreKey{Key: []float32{1, 2, 3, 4}},
        ClosestN:    3,
    })
    if err != nil {
        return err
    }
    fmt.Println("GetSimN Results:")
    for _, entry := range resp.Entries {
        fmt.Println(" - Key:", entry.Key.Key, "Value:", entry.Value.Value)
    }
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


    if err := client.exampleGetSimN(); err != nil {
        log.Fatalf("GetSimN failed: %v", err)
    }
}

```

</details>

- **Store** – The request targets `my_store`, which must already exist.

- **SearchInput** – A vector `[1, 2, 3, 4]` is used as the query input. This must match the dimensionality of the store.

- **ClosestN** – The request asks for the 3 most similar vectors.

- **Response** – The server returns the top matches as `resp.Entries`, including both the stored vectors and any metadata associated with them.

This makes `GetSimN` a fundamental query for retrieving entries most similar to a given embedding while leveraging the similarity algorithms supported by Ahnlich DB.
