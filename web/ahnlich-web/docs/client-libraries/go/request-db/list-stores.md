---
title: List Stores
---

# List Stores

## Description

`ListStores` returns the set of vector stores currently available on the Ahnlich DB server. Use it to **discover** what stores exist and to **validate** that a target store is present before you attempt writes or similarity queries.

## Behavior

- The call returns a collection of store metadata from the server at the time of the request. Each store entry includes: name, entry count, size in bytes, and non-linear index configurations (HNSW parameters or k-d tree) if any are active.

- Treat the result as an **enumeration**: don’t assume ordering or uniqueness semantics beyond what the server provides.

- An **empty list** simply means no stores are present yet on that server.

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


func (c *ExampleDBClient) Close() error { return c.conn.Close() }


// InfoServer example
func (c *ExampleDBClient) exampleInfoServer() error {
    resp, err := c.client.InfoServer(c.ctx, &dbquery.InfoServer{})
    if err != nil {
        return err
    }
    fmt.Println("InfoServer:", resp)
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


    if err := client.exampleInfoServer(); err != nil {
        log.Fatalf("InfoServer failed: %v", err)
    }
}

```

</details>

This method sends a `ListStores` request and prints the returned set of stores via `resp.Stores`. If the RPC fails, the returned error indicates the call did not succeed.
