---
title: Upsert
---

# Upsert

## Schema

This request accepts an optional `schema` field. When it is omitted, the server uses the `public` schema. Set `schema` to target a store in another schema.

## Description

The `Upsert` request updates a single entry matching a predicate condition. It errors if the predicate matches 0 or multiple entries.

Fields:
- `Store` - store name
- `Condition` - predicate that must match exactly one entry
- `NewKey` (optional) - new vector to replace existing key
- `NewValue` (optional) - metadata to update
- `MergeMetadata` - if true, merges new metadata into existing (default: false replaces entirely)
- `Schema` (optional) - schema namespace

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
    predicates "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/predicates"
)

const ServerAddr = "127.0.0.1:1369"

func stringPtr(value string) *string { return &value }

func main() {
    ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
    defer cancel()

    conn, err := grpc.DialContext(ctx, ServerAddr, grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithBlock())
    if err != nil {
        log.Fatalf("failed to dial: %v", err)
    }
    defer conn.Close()

    client := dbsvc.NewDBServiceClient(conn)

    condition := &predicates.PredicateCondition{
        Kind: &predicates.PredicateCondition_Value{
            Value: &predicates.Predicate{
                Kind: &predicates.Predicate_Equals{
                    Equals: &predicates.Equals{
                        Key: "id",
                        Value: &metadata.MetadataValue{
                            Value: &metadata.MetadataValue_RawString{RawString: "123"},
                        },
                    },
                },
            },
        },
    }

    newValue := &keyval.StoreValue{
        Value: map[string]*metadata.MetadataValue{
            "status": {Value: &metadata.MetadataValue_RawString{RawString: "published"}},
        },
    }

    resp, err := client.Upsert(ctx, &dbquery.Upsert{
        Store:         "my_store",
        Schema:        stringPtr("analytics"),
        Condition:     condition,
        NewValue:      newValue,
        MergeMetadata: true,
    })
    if err != nil {
        log.Fatalf("Upsert failed: %v", err)
    }
    fmt.Printf("Updated: %d\n", resp.Upsert.Updated)
}
```

</details>

This updates a single entry where `id = "123"`, merging the new status into existing metadata.
