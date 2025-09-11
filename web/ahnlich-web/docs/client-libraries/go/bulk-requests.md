---
title: Bulk Requests
sidebar_position: 4
---

# Bulk Requests — DB Pipeline

## Description

In scenarios where multiple operations need to be performed against the database in a single workflow, executing each request independently can introduce unnecessary network overhead and increase response time. To address this, the DB service provides a pipeline mechanism, which allows clients to bundle several queries together and send them as one request. This improves efficiency, ensures that queries are executed in a defined sequence, and guarantees that responses are returned in the same order as the submitted queries. Pipelines are especially useful in workloads where **set-and-retrieve** or **batch query patterns** are common.

## Source Code Example

```go

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

## Behavior

- Pipelines minimize network calls by sending multiple queries in a single request.

- Queries are executed in **order**, and results are returned in the same sequence.

- Both successful responses and errors are included, aligned with their respective queries.

- This approach optimizes performance and makes workflows more efficient when dealing with multiple dependent queries.

# Bulk Requests — AI Pipeline

Just like the DB service, the AI service also supports **pipelines** for combining multiple operations into a single request. This mechanism is particularly useful when you want to both **insert embeddings** and **query for similarities** in one sequence. By batching operations together, pipelines reduce **network latency**, improve throughput, and ensure that results are returned in the same order as the queries were issued.
This is valuable in AI workflows where you might first store **embeddings** (e.g., text, image, or other vectorized data) and immediately **retrieve the most similar entries** from the store.

## Source Code Example

```go
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

## Behavior and expectations

- Both `Set` and `GetSimN` are executed as part of the **same pipeline request**.

- Responses are returned in the **same sequence** as the queries (`Set - GetSimN`).

- If one query fails, its error is captured in the response while others continue execution.

- Pipelines improve efficiency by minimizing round-trips to the AI server.

- Useful in workflows where embeddings are created or updated and then immediately used for **similarity search**.

## Development & Testing

The Go SDK provides a set of Makefile targets and CI/CD workflows to simplify development, testing, and release management. These commands are primarily used by developers working on the SDK itself (rather than SDK consumers).

### Local Development Commands

### Install Dependencies

`make install-dependencies`

- Installs all required Go modules and external tools needed to build and test the SDK.

### Format Code

`make format`

- Runs formatting tools to enforce a consistent code style across the codebase.

### Run Tests (Sequential)

`make test`

- Executes all unit and integration tests sequentially to validate correctness of implementations.

### Lint Check

`make lint-check`

- Runs static analysis tools to ensure code quality, style consistency, and to catch potential issues before release.

## Deploy to GitHub Releases

The SDK uses GitHub Actions CI/CD to automate release publishing. Deployment steps:

1. **Bump version** in go.mod to reflect the new release.

**Create a Git tag** using semantic versioning format:

`git tag vX.Y.Z`
`git push origin vX.Y.Z`

Pushing the tag triggers the **CI/CD pipeline**, which:

- Builds the SDK

- Runs all tests

- Publishes the release artifacts to GitHub Releases
