---
title: Delete Key
---

# Delete Key

## Description

The `DeleteKey` request removes one or more vector entries from a store by explicitly specifying their keys. Unlike `DropStore`, which deletes the entire store, `DeleteKey` provides **granular control**, allowing you to delete only specific embeddings while keeping the rest of the store intact.

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


// -------------------- Delete Key --------------------
func (c *ExampleDBClient) exampleDeleteKey() error {
    _, err := c.client.DelKey(c.ctx, &dbquery.DelKey{
        Store: "my_stores",
        Keys:  []*keyval.StoreKey{{Key: []float32{1, 2, 3, 4}}},
    })
    if err != nil {
        return err
    }
    fmt.Println("Deleted key from store: my_stores")
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


    if err := client.exampleDeleteKey(); err != nil {
        log.Fatalf("DeleteKey failed: %v", err)
    }
}
```

</details>
