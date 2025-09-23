---
title: Rust
sidebar_position: 20
---

# Rust SDK

Build Ahnlich Applications with the Rust SDK – Vector Storage, Search, and AI tooling.

## [Rust-Specific Resources](/docs/client-libraries/rust/rust-specific-resources)
* Build Ahnlich Applications with the Rust SDK

* Ahnlich Rust Technical Resources

* Rust SDK Quickstart – Setup Guide 

## Overview

### Clients
* `db` — Vector Database client

* `ai` — AI Service client

### Pipelines
* Multiple ordered operations in batch

* Sequential execution, consistent reads

* Reduced gRPC round-trips

## What’s Included in the SDK

### Capabilities

#### DB Client
* Store vectors and metadata

* Query nearest neighbors with filters

* Manage stores  

#### AI Client
* Generate embeddings from raw inputs

* Interpret embeddings for similarity/clustering

* Complement DB client

### Typical Workflow

* AI Service → Generate embeddings

* DB Service → Store embeddings

* Response → Ordered results

## [Request – DB (Detailed)](/docs/client-libraries/rust/request-db)
### [Ping](/docs/client-libraries/rust/request-db/ping)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [Info Server](/docs/client-libraries/rust/request-db/info-server)

* Description

* Source Code Example

* Parameters

* Behavior

* When to use

### [List Stores](/docs/client-libraries/rust/request-db/list-stores)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [Create Store](/docs/client-libraries/rust/request-db/create-store)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [Set](/docs/client-libraries/rust/request-db/set)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [Get Sim N](/docs/client-libraries/rust/request-db/get-simn)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [Get Key](/docs/client-libraries/rust/request-db/get-key)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [Get by Predicate](/docs/client-libraries/rust/request-db/get-by-predicate)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [Create Predicate Index](/docs/client-libraries/rust/request-db/create-predicate-index)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [Drop Predicate Index](/docs/client-libraries/rust/request-db/drop-predicate-index)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [Delete Key](/docs/client-libraries/rust/request-db/delete-key)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [Drop Store](/docs/client-libraries/rust/request-db/drop-store)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [List Connected Clients](/docs/client-libraries/rust/request-db/list-connected-clients)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [Create Non-Linear Algorithm Index](/docs/client-libraries/rust/request-db/create-non-linear-algx)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [Drop Non-Linear Algorithm Index](/docs/client-libraries/rust/request-db/drop-non-linear-algx)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [Delete By Predicate](/docs/client-libraries/rust/request-db/delete-by-predicate)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

## [Request – AI (Detailed)](/docs/client-libraries/rust/request-ai)
### [Ping](/docs/client-libraries/rust/request-ai/ping)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [Info Server](/docs/client-libraries/rust/request-ai/info-server)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [List Stores](/docs/client-libraries/rust/request-ai/list-stores)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [Create Store](/docs/client-libraries/rust/request-ai/create-store)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [Set](/docs/client-libraries/rust/request-ai/set)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [Get Sim N](/docs/client-libraries/rust/request-ai/get-simn)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [Get Key](/docs/client-libraries/rust/request-ai/get-key)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [Get by Predicate](/docs/client-libraries/rust/request-ai/get-by-predicate)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [Create Predicate Index](/docs/client-libraries/rust/request-ai/create-predicate-index)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [Drop Predicate Index](/docs/client-libraries/rust/request-ai/drop-predicate-index)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [Delete Key](/docs/client-libraries/rust/request-ai/delete-key)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [Drop Store](/docs/client-libraries/rust/request-ai/drop-store)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [List Connected Clients](/docs/client-libraries/rust/request-ai/list-connected-clients)

* Description

* Source Code Example

* Returns

* Behavior

### [Create Non-Linear Algorithm Index](/docs/client-libraries/rust/request-ai/create-non-linear-algx)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [Drop Non-Linear Algorithm Index](/docs/client-libraries/rust/request-ai/drop-non-linear-algx)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [New](/docs/client-libraries/rust/request-ai/new)

* Description

* Source Code Example

* Parameters

* Returns

* Behavior

### [Purge Stores](/docs/client-libraries/rust/request-ai/purge-stores)

* Description

* Source Code Example

* Returns

* Behavior  

## [Pipeline](/docs/client-libraries/rust/pipeline)

* Description  

* Source Code Example  

* Parameters  

* Returns  

* Behavior

## [Types & Utilities](/docs/client-libraries/rust/types-and-utilities)

* Description 

* Usage  

* Details

## [Testing](/docs/client-libraries/rust/testing)

* Description  

* Key Test Scenarios  

* Purpose

## [Distributed Tracing](/docs/client-libraries/rust/distributed-tracing)

* Description  

* Usage  

* Benefits
