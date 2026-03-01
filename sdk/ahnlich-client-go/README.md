# Ahnlich Go Client SDK

[![Ahnlich TestSuite](https://github.com/deven96/ahnlich/actions/workflows/test.yml/badge.svg)](https://github.com/deven96/ahnlich/actions/workflows/test.yml)
[![Go Client Tag & Deploy](https://github.com/deven96/ahnlich/actions/workflows/go_tag_and_deploy.yml/badge.svg)](https://github.com/deven96/ahnlich/actions/workflows/go_tag_and_deploy.yml)

## Table of Contents

- [Installation](#installation)
- [Package Information](#package-information)
- [Server Response](#server-response)
- [Initialization](#initialization)
- [Connection Pooling](#connection-pooling)
- [Requests - DB](#requests---db)
  - [Ping](#ping)
  - [Info Server](#info-server)
  - [List Stores](#list-stores)
  - [Get Store](#get-store)
  - [Create Store](#create-store)
  - [Set](#set)
  - [Get Sim N](#get-sim-n)
  - [Get Key](#get-key)
  - [Get By Predicate](#get-by-predicate)
  - [Create Predicate Index](#create-predicate-index)
  - [Drop Predicate Index](#drop-predicate-index)
  - [Delete Key](#delete-key)
  - [Drop Store](#drop-store)
  - [List Connected Clients](#list-connected-clients)
  - [Create Non Linear Algorithm Index](#create-non-linear-algorithm-index)
  - [Drop Non Linear Algorithm Index](#drop-non-linear-algorithm-index)
  - [Delete Predicate](#delete-predicate)

- [Requests - AI](#requests---ai)
  - [Ping](#ping-1)
  - [Info Server](#info-server-1)
  - [List Stores](#list-stores-1)
  - [Get Store](#get-store-1)
  - [Create Store](#create-store-1)
  - [Set](#set-1)
  - [Get Sim N](#get-sim-n-1)
  - [Get By Predicate](#get-by-predicate-1)
  - [Create Predicate Index](#create-predicate-index-1)
  - [Drop Predicate Index](#drop-predicate-index-1)
  - [Delete Key](#delete-key-1)
  - [Drop Store](#drop-store-1)
  - [Create Non Linear Algorithm Index](#create-non-linear-algorithm-index-1)
  - [Drop Non Linear Algorithm Index](#drop-non-linear-algorithm-index-1)
- [Bulk Requests](#bulk-requests)
- [Development & Testing](#development--testing)
- [Deploy to GitHub Releases](#deploy-to-github-releases)
- [Type Meanings](#type-meanings)
- [Change Log](#change-log)

## Installation

Using Go modules:
```bash
go get github.com/deven96/ahnlich/sdk/ahnlich-client-go@vX.Y.Z
```
or add to your `go.mod`:
```go
require (
    github.com/deven96/ahnlich/sdk/ahnlich-client-go vX.Y.Z
)
```

## Package Information

This module provides:
- gRPC service stubs for DB and AI
- Pipeline utilities for batching RPC calls

## Server Response

All DB query/response types live under:
```go
import "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/db"
```
All AI query/response types live under:
```go
import "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai"
```

## Initialization

### Client

#### DB Client
```go
package example
import (
    "context"
    "fmt"
    "log"
    "time"
    "google.golang.org/grpc"
    "google.golang.org/grpc/credentials/insecure"

    algorithm "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/algorithm/algorithms"
    dbsvc     "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/services/db_service"
    dbquery   "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/db/query"
    keyval    "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/keyval"
    metadata  "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/metadata"
    predicates "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/predicates"
)

const ServerAddr = "127.0.0.1:1369"

// ExampleDBClient holds the gRPC connection and client.
type ExampleDBClient struct {
    conn   *grpc.ClientConn
    client dbsvc.DBServiceClient
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
    return &ExampleDBClient{conn: conn, client: client}, nil
}

// Close closes the gRPC connection.
func (c *ExampleDBClient) Close() error {
    return c.conn.Close()
}
```

#### AI Client
```go
import (
    "context"
    "fmt"
    "time"
    "google.golang.org/grpc"
    "google.golang.org/grpc/credentials/insecure"

    algorithm "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/algorithm/algorithms"
    aisvc      "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/services/ai_service"
    aiquery    "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/query"
    aimodel    "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/models"
    keyval     "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/keyval"
    metadata   "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/metadata"
    preprocess "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/preprocess"
    predicates "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/predicates"
)

const AIAddr = "127.0.0.1:1370"

// ExampleAIClient holds the gRPC connection and AI client.
type ExampleAIClient struct {
    conn   *grpc.ClientConn
    client aisvc.AIServiceClient
}

// NewAIClient connects to the Ahnlich AI server.
func NewAIClient(ctx context.Context) (*ExampleAIClient, error) {
    conn, err := grpc.DialContext(
        ctx,
        AIAddr,
        grpc.WithTransportCredentials(insecure.NewCredentials()),
        grpc.WithBlock(),
    )
    if err != nil {
        return nil, fmt.Errorf("failed to dial AI server %q: %w", AIAddr, err)
    }
    client := aisvc.NewAIServiceClient(conn)
    return &ExampleAIClient{conn: conn, client: client}, nil
}

// Close closes the gRPC connection.
func (c *ExampleAIClient) Close() error {
    return c.conn.Close()
}
```

## Connection Pooling

> Go’s gRPC client reuses connections under the hood. For advanced pooling, wrap the `*grpc.ClientConn` accordingly.

## Requests - DB

### Ping
```go
func (c *ExampleDBClient) examplePingDB() error {
    resp, err := c.client.Ping(c.ctx, &dbquery.Ping{})
    if err != nil { return err }
    fmt.Println("Ping:", resp)
    return nil
}
```

### Info Server
```go
func (c *ExampleDBClient) exampleInfoServer() error {
    resp, err := c.client.InfoServer(c.ctx, &dbquery.InfoServer{})
    fmt.Println("InfoServer:", resp)
    return err
}
```

### List Stores
```go
func (c *ExampleDBClient) exampleListStores() error {
    resp, err := c.client.ListStores(c.ctx, &dbquery.ListStores{})
    if err != nil { return err }
    for _, s := range resp.Stores {
        // Each StoreInfo includes Name, Len, SizeInBytes, NonLinearIndices,
        // PredicateIndices ([]string), and Dimension (uint32).
        fmt.Printf("Store: %s, Dimension: %d, PredicateIndices: %v\n",
            s.Name, s.Dimension, s.PredicateIndices)
    }
    return nil
}
```

### Get Store
```go
func (c *ExampleDBClient) exampleGetStore() error {
    resp, err := c.client.GetStore(c.ctx, &dbquery.GetStore{Store: "my_store"})
    if err != nil { return err }
    // resp is a *StoreInfo with Name, Len, SizeInBytes, NonLinearIndices,
    // PredicateIndices ([]string), and Dimension (uint32).
    fmt.Printf("Store: %s, Dimension: %d, PredicateIndices: %v\n",
        resp.Name, resp.Dimension, resp.PredicateIndices)
    return nil
}
```

### Create Store
```go
func (c *ExampleDBClient) exampleCreateStore() error {
    _, err := c.client.CreateStore(c.ctx, &dbquery.CreateStore{Store:"my_store", Dimension:4})
    return err
}
```

### Set
```go
func (c *ExampleDBClient) exampleSet() error {
    entries := []*keyval.DbStoreEntry{{Key:&keyval.StoreKey{Key:[]float32{1,2,3,4}},Value:&keyval.StoreValue{Value:map[string]*metadata.MetadataValue{"label":{Value:&metadata.MetadataValue_RawString{RawString:"A"}}}}}}
    _, err := c.client.Set(c.ctx, &dbquery.Set{Store:"my_store", Inputs:entries})
    return err
}
```

### Get Sim N
```go
func (c *ExampleDBClient) exampleGetSimN() error {
    resp, err := c.client.GetSimN(c.ctx, &dbquery.GetSimN{Store:"my_store", SearchInput:&keyval.StoreKey{Key:[]float32{1,2,3,4}}, ClosestN:3})
    fmt.Println("GetSimN:", resp.Entries)
    return err
}
```

### Get Key
```go
func (c *ExampleDBClient) exampleGetKey() error {
    resp, err := c.client.GetKey(c.ctx, &dbquery.GetKey{Store:"my_store", Keys:[]*keyval.StoreKey{{Key:[]float32{1,2,3,4}}}})
    fmt.Println("GetKey:", resp.Entries)
    return err
}
```

### Get By Predicate
```go
func (c *ExampleDBClient) exampleGetByPredicate() error {
    cond := &predicates.PredicateCondition{Kind:&predicates.PredicateCondition_Value{Value:&predicates.Predicate{Kind:&predicates.Predicate_Equals{Equals:&predicates.Equals{Key:"label",Value:&metadata.MetadataValue{Value:&metadata.MetadataValue_RawString{RawString:"A"}}}}}}}
    resp, err := c.client.GetByPredicate(c.ctx, &dbquery.GetByPredicate{Store:"my_store", Condition:cond})
    fmt.Println("GetByPredicate:", resp.Entries)
    return err
}
```

### Create Predicate Index
```go
func (c *ExampleDBClient) exampleCreatePredicateIndex() error {
    _, err := c.client.CreatePredicateIndex(c.ctx, &dbquery.CreatePredIndex{Store:"my_store", Predicates:[]string{"label"}})
    return err
} 
```

### Drop Predicate Index
```go
func (c *ExampleDBClient) exampleDropPredicateIndex() error {
    _, err := c.client.DropPredicateIndex(c.ctx, &dbquery.DropPredIndex{Store:"my_store", Predicates:[]string{"label"}, ErrorIfNotExists:true})
    return err
}
```

### Delete Key
```go
func (c *ExampleDBClient) exampleDeleteKey() error {
    _, err := c.client.DelKey(c.ctx, &dbquery.DelKey{Store:"my_store", Keys:[]*keyval.StoreKey{{Key:[]float32{1,2,3,4}}}})
    return err
}
```

### Drop Store
```go
func (c *ExampleDBClient) exampleDropStore() error {
    _, err := c.client.DropStore(c.ctx, &dbquery.DropStore{Store: "my_store"})
    return err
}
```

### List Connected Clients
```go
func (c *ExampleDBClient) exampleListConnectedClients() error {
    resp, err := c.client.ListClients(c.ctx, &dbquery.ListClients{})
    if err != nil {
        return err
    }
    fmt.Println("Connected Clients:", resp.Clients)
    return nil
}
```

### Create Non Linear Algorithm Index
```go
import (
    nonlinear "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/algorithm/nonlinear"
)

func (c *ExampleDBClient) exampleCreateNonLinearAlgoIndex() error {
    // Create a KDTree index
    _, err := c.client.CreateNonLinearAlgorithmIndex(c.ctx, &dbquery.CreateNonLinearAlgorithmIndex{
        Store: "my_store",
        NonLinearIndices: []*nonlinear.NonLinearIndex{
            {Index: &nonlinear.NonLinearIndex_Kdtree{Kdtree: &nonlinear.KDTreeConfig{}}},
        },
    })
    if err != nil { return err }

    // Or create an HNSW index (with optional config)
    _, err = c.client.CreateNonLinearAlgorithmIndex(c.ctx, &dbquery.CreateNonLinearAlgorithmIndex{
        Store: "my_store",
        NonLinearIndices: []*nonlinear.NonLinearIndex{
            {Index: &nonlinear.NonLinearIndex_Hnsw{Hnsw: &nonlinear.HNSWConfig{}}},
        },
    })
    return err
}
```

### Drop Non Linear Algorithm Index
```go
func (c *ExampleDBClient) exampleDropNonLinearAlgoIndex() error {
    _, err := c.client.DropNonLinearAlgorithmIndex(c.ctx, &dbquery.DropNonLinearAlgorithmIndex{
        Store:            "my_store",
        NonLinearIndices: []nonlinear.NonLinearAlgorithm{nonlinear.NonLinearAlgorithm_KDTree},
        ErrorIfNotExists: true,
    })
    return err
}
```

### Delete Predicate
```go
func (c *ExampleDBClient) exampleDeletePredicate() error {
    cond := &predicates.PredicateCondition{
        Kind: &predicates.PredicateCondition_Value{Value: &predicates.Predicate{
            Kind: &predicates.Predicate_Equals{Equals: &predicates.Equals{
                Key:   "label",
                Value: &metadata.MetadataValue{Value: &metadata.MetadataValue_RawString{RawString: "A"}},
            }},
        }},
    }
    _, err := c.client.DelPred(c.ctx, &dbquery.DelPred{
        Store:     "my_store",
        Condition: cond,
    })
    return err
}
```

## Requests - AI

### Ping
```go
func (c *ExampleAIClient) examplePingAI(ctx context.Context) error {
    resp, err := c.client.Ping(c.ctx, &aiquery.Ping{})
    if err != nil {
        return err
    }
    fmt.Println("AI Ping:", resp)
    return nil
}
```

### Info Server
```go
func (c *ExampleAIClient) exampleInfoServerAI(ctx context.Context) error {
    resp, err := c.client.InfoServer(c.ctx, &aiquery.InfoServer{})
    if err != nil {
        return err
    }
    fmt.Println("AI InfoServer:", resp)
    return nil
}
```

### List Stores
```go
func (c *ExampleAIClient) exampleListStoresAI(ctx context.Context) error {
    resp, err := c.client.ListStores(c.ctx, &aiquery.ListStores{})
    if err != nil {
        return err
    }
    for _, s := range resp.Stores {
        // Each AIStoreInfo includes Name, QueryModel, IndexModel, EmbeddingSize,
        // PredicateIndices ([]string), and Dimension (uint32).
        fmt.Printf("Store: %s, QueryModel: %v, IndexModel: %v, EmbeddingSize: %d, Dimension: %d, PredicateIndices: %v\n",
            s.Name, s.QueryModel, s.IndexModel, s.EmbeddingSize, s.Dimension, s.PredicateIndices)
    }
    return nil
}
```

### Get Store
```go
func (c *ExampleAIClient) exampleGetStoreAI(ctx context.Context) error {
    resp, err := c.client.GetStore(c.ctx, &aiquery.GetStore{Store: "ai_store"})
    if err != nil { return err }
    // resp is a *AIStoreInfo with Name, QueryModel, IndexModel, EmbeddingSize,
    // PredicateIndices ([]string), and Dimension (uint32).
    fmt.Printf("Store: %s, QueryModel: %v, IndexModel: %v, EmbeddingSize: %d, Dimension: %d, PredicateIndices: %v\n",
        resp.Name, resp.QueryModel, resp.IndexModel, resp.EmbeddingSize, resp.Dimension, resp.PredicateIndices)
    return nil
}
```

### Create Store
```go
func (c *ExampleAIClient) exampleCreateStoreAI(ctx context.Context) error {
    _, err := c.client.CreateStore(c.ctx, &aiquery.CreateStore{
        Store:      "ai_store",
        QueryModel: aimodel.AIModel_ALL_MINI_LM_L6_V2,
        IndexModel: aimodel.AIModel_ALL_MINI_LM_L6_V2,
    })
    return err
}
```

### Set
```go
func (c *ExampleAIClient) exampleSetAI(ctx context.Context) error {
    inputs := []*keyval.AiStoreEntry{
        {
            Key: &keyval.StoreInput{Value: &keyval.StoreInput_RawString{RawString: "X"}},
            Value: &keyval.StoreValue{Value: map[string]*metadata.MetadataValue{
                "f": {Value: &metadata.MetadataValue_RawString{RawString: "v"}},
            }},
        },
    }
    _, err := c.client.Set(c.ctx, &aiquery.Set{
        Store:            "ai_store",
        Inputs:           inputs,
        PreprocessAction: preprocess.PreprocessAction_NoPreprocessing,
    })
    return err
}
```

### Get Sim N
```go
func (c *ExampleAIClient) exampleGetSimNAI(ctx context.Context) error {
    resp, err := c.client.GetSimN(c.ctx, &aiquery.GetSimN{
        Store:       "ai_store",
        SearchInput: &keyval.StoreInput{Value: &keyval.StoreInput_RawString{RawString: "X"}},
        ClosestN:    3,
    })
    if err != nil {
        return err
    }
    fmt.Println("AI GetSimN:", resp.Entries)
    return nil
}
```

### Get By Predicate
```go
func (c *ExampleAIClient) exampleGetByPredicateAI(ctx context.Context) error {
    cond := &predicates.PredicateCondition{
        Kind: &predicates.PredicateCondition_Value{Value: &predicates.Predicate{
            Kind: &predicates.Predicate_Equals{Equals: &predicates.Equals{
                Key:   "f",
                Value: &metadata.MetadataValue{Value: &metadata.MetadataValue_RawString{RawString: "v"}},
            }},
        }},
    }
    resp, err := c.client.GetByPredicate(c.ctx, &aiquery.GetByPredicate{
        Store:     "ai_store",
        Condition: cond,
    })
    if err != nil {
        return err
    }
    fmt.Println("AI GetByPredicate:", resp.Entries)
    return nil
}
```

### Create Predicate Index
```go
func (c *ExampleAIClient) exampleCreatePredicateIndexAI(ctx context.Context) error {
    _, err := c.client.CreatePredicateIndex(c.ctx, &aiquery.CreatePredicateIndex{
        Store:      "ai_store",
        Predicates: []string{"f"},
    })
    return err
}
```

### Drop Predicate Index
```go
func (c *ExampleAIClient) exampleDropPredicateIndexAI(ctx context.Context) error {
    _, err := c.client.DropPredicateIndex(c.ctx, &aiquery.DropPredicateIndex{
        Store:           "ai_store",
        Predicates:      []string{"f"},
        ErrorIfNotExists: true,
    })
    return err
}
```

### Delete Key
```go
func (c *ExampleAIClient) exampleDeleteKeyAI(ctx context.Context) error {
    _, err := c.client.DeleteKey(c.ctx, &aiquery.DeleteKey{
        Store: "ai_store",
        Keys:  []*keyval.StoreInput{{Value: &keyval.StoreInput_RawString{RawString: "X"}}},
    })
    return err
}
```

### Drop Store
```go
func (c *ExampleAIClient) exampleDropStoreAI(ctx context.Context) error {
    _, err := c.client.DropStore(c.ctx, &aiquery.DropStore{
        Store:           "ai_store",
        ErrorIfNotExists: true,
    })
    return err
}
```

### Create Non Linear Algorithm Index
```go
import (
    nonlinear "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/algorithm/nonlinear"
)

func (c *ExampleAIClient) exampleCreateNonLinearIndexAI(ctx context.Context) error {
    // Create a KDTree index
    _, err := c.client.CreateNonLinearAlgorithmIndex(c.ctx, &aiquery.CreateNonLinearAlgorithmIndex{
        Store: "ai_store",
        NonLinearIndices: []*nonlinear.NonLinearIndex{
            {Index: &nonlinear.NonLinearIndex_Kdtree{Kdtree: &nonlinear.KDTreeConfig{}}},
        },
    })
    if err != nil { return err }

    // Or create an HNSW index (with optional config)
    _, err = c.client.CreateNonLinearAlgorithmIndex(c.ctx, &aiquery.CreateNonLinearAlgorithmIndex{
        Store: "ai_store",
        NonLinearIndices: []*nonlinear.NonLinearIndex{
            {Index: &nonlinear.NonLinearIndex_Hnsw{Hnsw: &nonlinear.HNSWConfig{}}},
        },
    })
    return err
}
```

### Drop Non Linear Algorithm Index
```go
func (c *ExampleAIClient) exampleDropNonLinearIndexAI(ctx context.Context) error {
    _, err := c.client.DropNonLinearAlgorithmIndex(c.ctx, &aiquery.DropNonLinearAlgorithmIndex{
        Store:            "ai_store",
        NonLinearIndices: []nonlinear.NonLinearAlgorithm{nonlinear.NonLinearAlgorithm_KDTree},
        ErrorIfNotExists: true,
    })
    return err
}
```

## Bulk Requests

Use pipeline to combine multiple calls:

### DB Pipeline
```go
import (
    "context"
    "fmt"
    "time"
    "google.golang.org/grpc/credentials/insecure"
    "google.golang.org/grpc"


    pipeline "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/db/pipeline"
    dbquery  "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/db/query"
    dbsvc    "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/services/db_service"
    keyval   "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/keyval"
)

const ServerAddr = "127.0.0.1:1369"

func examplePipelineDB() error {
    c.ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
    defer cancel()

    conn, err := grpc.DialContext(c.ctx, proc.ServerAddr, grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithBlock())
    if err != nil {
        return fmt.Errorf("failed to dial DB server %q: %w", ServerAddr, err)
    }
    defer conn.Close()

    client := dbsvc.NewDBServiceClient(conn)

    // Build pipeline queries
    setQ := &pipeline.DBQuery{Query: &pipeline.DBQuery_Set{Set: &dbquery.Set{
        Store:  "my_store",
        Inputs: []*keyval.DbStoreEntry{/* ... */},
    }}}
    getQ := &pipeline.DBQuery{Query: &pipeline.DBQuery_GetKey{GetKey: &dbquery.GetKey{
        Store: "my_store",
        Keys:  []*keyval.StoreKey{/* ... */},
    }}}

    // Execute pipeline
    req := &pipeline.DBRequestPipeline{Queries: []*pipeline.DBQuery{setQ, getQ}}
    resp, err := client.Pipeline(c.ctx, req)
    if err != nil {
        return err
    }
    fmt.Println("Pipeline responses:", resp.Responses)
    return nil
}
```

### AI Pipeline
```go
import (
    "context"
    "fmt"
    "time"
    "google.golang.org/grpc/credentials/insecure"
    "google.golang.org/grpc"

    pipeline "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/pipeline"
    aisvc    "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/services/ai_service"
    aiquery  "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai/query"
    keyval   "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/keyval"
)

const ServerAddr = "127.0.0.1:1370"

func examplePipelineAI() error {
    c.ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
    defer cancel()

    conn, err := grpc.DialContext(c.ctx, proc.ServerAddr, grpc.WithTransportCredentials(insecure.NewCredentials()), grpc.WithBlock())
    if err != nil {
        return fmt.Errorf("failed to dial AI server %q: %w", ServerAddr, err)
    }
    defer conn.Close()

    client := aisvc.NewAIServiceClient(conn)

    // Build pipeline queries
    setQ := &pipeline.AIQuery{Query: &pipeline.AIQuery_Set{Set: &aiquery.Set{
        Store:  "ai_store",
        Inputs: []*keyval.AiStoreEntry{/* ... */},
    }}}
    simQ := &pipeline.AIQuery{Query: &pipeline.AIQuery_GetSimN{GetSimN: &aiquery.GetSimN{
        Store:       "ai_store",
        SearchInput: &keyval.StoreInput{Value: &keyval.StoreInput_RawString{RawString: "X"}},
        ClosestN:    3,
    }}}

    // Execute pipeline
    req := &pipeline.AIRequestPipeline{Queries: []*pipeline.AIQuery{setQ, simQ}}
    resp, err := client.Pipeline(c.ctx, req)
    if err != nil {
        return err
    }
    fmt.Println("AI pipeline responses:", resp.Responses)
    return nil
}
```

## Development & Testing

```bash
make install-dependencies
make generate    # regenerate Go protobuf code from proto definitions (requires buf)
make format
make test        # sequential
make lint-check
```

## Deploy to GitHub Releases

- Bump version in go.mod
- Create a git tag `vX.Y.Z`
- Push tag to trigger CI/CD release

## Type Meanings

- **StoreKey**: `[]float32` for DB, `StoreInput` for AI
- **StoreValue**: map of metadata fields
- **Predicates**: filter conditions for stored values
- **Pipeline**: batch RPC builder

## Change Log

| Version | Description                             |
|---------|-----------------------------------------|
| 0.1.0   | Initial Go SDK release                 |
