---
title: Go Specific Resources
sidebar_posiiton: 1
---

## QuickStart - Setup

Follow these steps to set up the Ahnlich Go SDK on your local machine.

## Install Go

<!-- Go to the official [Go website](https://go.dev/doc/install) -->

```go
  # Check if you already have Go installed:
  go version
```

_Example output:_

```go
  go version go1.20.5 linux/amd64
```

If Go is not installed, follow the steps for your operating system:

### macOS

```go
  # Install using Homebrew:
  brew install go
```

```go
  # Verify the installation:
  go version
```

### Windows

    1. Download the MSI installer from the official Go website.

    2. Run the installer and follow the setup wizard.

**Open Command Prompt or PowerShell and check:**

```go
  go version
```

### Linux

Download the Go tarball from the official Go website.

**Source Code Source Code Example:**

```go
  # Download Go
  wget https://go.dev/dl/go1.20.5.linux-amd64.tar.gz

  # Extract to /usr/local
  sudo tar -C /usr/local -xzf go1.20.5.linux-amd64.tar.gz

  # Add Go to your PATH (append this line to ~/.bashrc or ~/.zshrc)
  export PATH=$PATH:/usr/local/go/bin

  # Reload your shell (or run: source ~/.bashrc) and verify installation
  go version
```

## Install the Ahnlich Go SDK

The Ahnlich Go SDK provides the client libraries you’ll use to connect with Ahnlich DB (vector database) and Ahnlich AI (semantic embedding/search).

You can add it to your project in two ways:

```go
  Install with go get
```

Run the following command in your project directory:

```go
  Using go get
  github.com/deven96/ahnlich/sdk/ahnlich-client-go@vX.Y.Z
```

- Replace vX.Y.Z with the latest version of the SDK.

- This will install support for both DB and AI request/response types.

- After running it, you should see the SDK listed in your go.mod file.

Or Add it manually in **go.mod**

Open your go.mod file and add the following line inside the require block:

```go
  require (
      github.com/deven96/ahnlich/sdk/ahnlich-client-go vX.Y.Z
  )
```

- Again, replace vX.Y.Z with the desired version.

- Once added, **run:**

```go
  go mod tidy
```

This will download the SDK and clean up your module dependencies.

### What’s Included in the SDK?

The Ahnlich Go SDK contains everything you need to work with both the DB service and the AI service.
It provides strongly-typed request and response objects that you can use to interact with Ahnlich DB AND AI.

These types let you perform operations such as:

#### Request-DB for:

- Creating and deleting stores

- Inserting and updating vectors

- Running similarity search queries

- Managing filters and predicates for advanced search

#### Request-AI for:

- Generating vector embeddings from text or binary data

- Running embedding-based queries

- Using AI-powered similarity for enhanced search

#### With these request/response types, you can build Go applications that:

- Use **Ahnlich DB** for exact vector similarity search.

- Use **Ahnlich AI** for semantic embeddings and intelligent search.

- Or combine both for hybrid workflows.

Once installed, you can start building Go applications that use Ahnlich DB for vector search and Ahnlich AI for semantic embeddings.

### Package Information

This module provides:

- **gRPC service** stubs for both **DB** and **AI**.

- **Pipeline utilities** for batching RPC calls efficiently.

#### DB gRPC types

All DB request/response messages live under:

_Example_

```go
  import "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/db"
```

- These cover operations like:

  ... Creating and deleting stores

  ... Inserting and updating vectors

  ... Running similarity searches

#### AI gRPC types

All AI request/response messages live under:

```go
  import "github.com/deven96/ahnlich/sdk/ahnlich-client-go/grpc/ai"
```

- These cover operations like:

  ... Embedding text or binary data

  ... Querying embeddings

  ... Powering semantic search

- Pipeline Utilities
  Utilities for **batching RPC calls**, allowing efficient handling of multiple requests in a single operation.

## Initialization

The Ahnlich Go SDK provides client implementations for both **DB** and **AI** services.
Before issuing queries or embedding requests, you must first initialize a client connection over gRPC.

### Client Setup

Both DB and AI clients share a similar initialization pattern:

- Define the **server address** (default `DB: 127.0.0.1:1369`, `AI: 127.0.0.1:1370`).

- Use `grpc.DialContext` with insecure credentials for local development.

- Create a new service client from the generated gRPC stubs.

- Always close the client connection when finished.

This ensures proper resource management and clean shutdown.

### DB Client

The DB client allows you to connect to an Ahnlich DB instance and perform vector operations such as creating stores, inserting embeddings, or querying for similarity.
package example

<details>
  <summary>Click to expand source code</summary>

```go
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
</details>

### AI Client

The AI client connects to the Ahnlich AI service, which handles embedding generation and semantic queries.

<details>
  <summary>Click to expand source code</summary>

```go
package example

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
</details>

### Connection Pooling

Go’s gRPC client reuses connections under the hood. This means you can safely create multiple clients from a single connection without additional overhead.

For applications that require higher throughput or advanced resource control, you can manage your own pool of gRPC connections. This allows you to balance requests across multiple servers and fine-tune concurrency limits.
