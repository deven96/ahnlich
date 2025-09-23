---
title: List Connected Clients
---

# List Connected Clients

## Description

The `ListConnectedClients` request allows you to query the database server for all currently connected clients. This is particularly useful for **monitoring, debugging,** and ensuring that multiple services or developers interacting with the same Ahnlich DB instance are tracked properly.

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


func (c *ExampleDBClient) exampleListConnectedClients() error {
    resp, err := c.client.ListClients(c.ctx, &dbquery.ListClients{})
    if err != nil {
        return err
    }
    fmt.Println("Connected Clients:", resp.Clients)
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


    if err := client.exampleListConnectedClients(); err != nil {
        log.Fatalf("ListConnectedClients failed: %v", err)
    }
}
```

</details>

This method requests the creation of a store named `"my_store"` with a vector dimensionality of 4. The response is ignored in this example, but the absence of an error indicates success.
