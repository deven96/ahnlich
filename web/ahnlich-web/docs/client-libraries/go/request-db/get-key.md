---
title: GetKey
---

# GetKey

## Description

The GetKey request is used to retrieve **specific entries** from a store by providing the exact vector keys. Unlike GetSimN, which searches for approximate or closest matches, GetKey performs a **direct lookup** based on the stored vectors.

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


// -------------------- GetKey --------------------
func (c *ExampleDBClient) exampleGetKey() error {
    resp, err := c.client.GetKey(c.ctx, &dbquery.GetKey{
        Store: "my_stores",
        Keys:  []*keyval.StoreKey{{Key: []float32{1, 2, 3, 4}}},
    })
    if err != nil {
        return err
    }
    fmt.Println("GetKey Results:")
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


    if err := client.exampleGetKey(); err != nil {
        log.Fatalf("GetKey failed: %v", err)
    }
}
```

</details>

- **Store** – Targets the `my_store` vector store.

- **Keys** – A slice of vector keys to look up. In this case, the request asks for the entry with key `[1, 2, 3, 4]`.

- **Response** – If the key exists in the store, the server returns the corresponding entries (vector + metadata) inside `resp.Entries`.

This operation is especially useful when you need **exact retrieval** of vectors and their metadata, such as fetching embeddings for validation, re-indexing, or checking consistency of stored data.
