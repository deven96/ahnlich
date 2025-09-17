---
title: Info Server
sidebar_posiiton: 2
---

# Info Server

## Description

The `InfoServer` request retrieves metadata about the running Ahnlich DB server.

Whereas `Ping` only checks connectivity, `InfoServer` provides richer diagnostic details. It allows developers and operators to understand the state and configuration of the server they are connected to.

Common use cases include:

- **Cluster diagnostics** – Retrieve server-specific information such as version, uptime, or available features.

- **Environment verification** – Confirm that the client is connected to the intended server instance (useful in multi-node or distributed deployments).

- **Monitoring and logging** – Integrate server details into dashboards or log pipelines to track health and consistency.

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

This method defines exampleInfoServer on the ExampleDBClient type. It calls the InfoServer RPC and prints the server’s response. Any error encountered during the call is returned to the caller.
