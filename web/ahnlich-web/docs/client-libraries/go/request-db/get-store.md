---
title: Get Store
---

# Get Store

## Description

The `GetStore` request retrieves detailed information about a specific store by name. This is useful for inspecting store configuration, checking dimensions, or monitoring size.

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

func (c *ExampleDBClient) exampleGetStore() error {
    resp, err := c.client.GetStore(c.ctx, &dbquery.GetStore{
        Store: "my_store",
    })
    if err != nil {
        return err
    }

    fmt.Printf("Store name: %s\n", resp.Name)
    fmt.Printf("Number of entries: %d\n", resp.Len)
    fmt.Printf("Size in bytes: %d\n", resp.SizeInBytes)
    fmt.Printf("Dimension: %d\n", resp.Dimension)
    fmt.Printf("Predicate indices: %v\n", resp.PredicateIndices)
    fmt.Printf("Non-linear indices: %v\n", resp.NonLinearIndices)

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

    if err := client.exampleGetStore(); err != nil {
        log.Fatalf("GetStore failed: %v", err)
    }
}
```

</details>

## Parameters

| Parameter | Type | Required | Description |
|-----------|------|----------|-------------|
| `Store` | `string` | Yes | The name of the store to retrieve |

## Response: StoreInfo

| Field | Type | Description |
|-------|------|-------------|
| `Name` | `string` | Store name |
| `Len` | `uint64` | Number of entries in the store |
| `SizeInBytes` | `uint64` | Total size of the store in bytes |
| `Dimension` | `uint32` | Vector dimension |
| `PredicateIndices` | `[]string` | List of indexed predicate keys |
| `NonLinearIndices` | `[]*NonLinearIndex` | List of non-linear algorithm indices |

## Notes

- Returns an error if the store does not exist
- Use `ListStores` to get information about all stores
- The `SizeInBytes` field is useful for monitoring memory usage
