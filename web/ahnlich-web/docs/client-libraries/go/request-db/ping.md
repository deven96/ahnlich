---
title: Ping
sidebar_posiiton: 1
---

# Ping

## Description

The `Ping` request is the simplest way to verify connectivity between your Go client and an active **Ahnlich DB server**.

When a client calls `Ping`, the server responds immediately if it is reachable and healthy. This is useful in a variety of scenarios:

- **Connection health check** – Before performing more expensive operations such as storing or searching vectors, you can ensure the server is running and ready to process requests.

- **Service monitoring** – Can be integrated into a heartbeat system to periodically confirm the DB server is online.

- **Debugging setup** – Helpful for quickly confirming that your gRPC connection and server configuration are correct.

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


      dbsvc "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/services/db_service"
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


func (c *ExampleDBClient) examplePingDB() error {
    resp, err := c.client.Ping(c.ctx, &dbquery.Ping{})
    if err != nil {
        return err
    }
    fmt.Println("Ping:", resp)
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


    if err := client.examplePingDB(); err != nil {
        log.Fatalf("Ping failed: %v", err)
    }
}
```

</details>

This example defines an `examplePingDB` method on the `ExampleDBClient` type. It sends a `Ping` request to the DB service using the `dbquery.Ping{}` message and prints the response. If the server is unavailable or the request fails, an error is returned.
