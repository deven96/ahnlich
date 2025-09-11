---
title: Set
---

# Set

## Description

The `Set` request is used to **insert or update entries** (vectors with associated metadata) into a given store. This is one of the core operations of Ahnlich DB, as it establishes the embeddings that can later be queried for similarity search.

Each entry in a store consists of two parts:

1. **Key** – the vector itself (a fixed-dimension embedding).

2. **Value** – metadata stored alongside the vector, represented as a key-value map. Metadata enables additional filtering and querying beyond pure vector similarity.

## Behavior

- The vector provided in `StoreKey` **must match the store’s dimension**. For example, a 4-dimensional store only accepts `[1, 2, 3, 4]` shaped vectors.

- Metadata values can be of different types (strings, numbers, booleans). In the example, a string label `"A"` is stored.

- Multiple entries can be inserted in a single request by batching them in the `Inputs` field.

- If a key already exists in the store, calling Set again will **update** its value.

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
    metadata "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/metadata"
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


// Set example
func (c *ExampleDBClient) exampleSet(store string) error {
    entries := []*keyval.DbStoreEntry{
        {
            Key: &keyval.StoreKey{Key: []float32{1, 2, 3, 4}},
            Value: &keyval.StoreValue{
                Value: map[string]*metadata.MetadataValue{
                    "label": {
                        Value: &metadata.MetadataValue_RawString{RawString: "A"},
                    },
                },
            },
        },
    }
    _, err := c.client.Set(c.ctx, &dbquery.Set{Store: store, Inputs: entries})
    if err != nil {
        return err
    }
    fmt.Println("Inserted entry into store:", store)
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


    storeName := "my_store"
    if err := client.exampleSet(storeName); err != nil {
        log.Fatalf("Set failed: %v", err)
    }
}

```

</details>

This inserts a single vector `[1, 2, 3, 4]` into the store `my_store` with metadata `{ "label": "A" }`.
