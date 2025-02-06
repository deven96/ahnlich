
# [RFC] GRPC Protocol For Ahnlich

Summary: Swap out current ahnlich protocol for  GRPC 

* Created: Jan 27, 2025
* Last Updated: Jan 27, 2025
* Current Version: 1.0.0
* Author: [David](https://github.com/Iamdavidonuh)
* Contributors: [Diretnan](https://github.com/deven96)

---

This RFC aims to provide a migration plan for ahnlich’s current protocol to GRPC. It explains the reasoning behind this decision while providing a comprehensive overview of the current implementation, strengths and it’s limitations

## Background
Before the migration plan, some backstory is needed. Ahnlich utilizes the binary protocol for its communication over TCP. It utilizes Bincode for serialization and deserialization of objects passed over a network. Client libraries serve as wrappers to programmatically interact with the ahnlich servers. 

Currently, we have python and rust clients that implements the ahnlich spec using bincode serialization to be able to communicate with the ahnlich servers. The idea is to have clients in different and any language and therein lies the problem. Because of the limited support of Bincode across multiple languages, we are confined to the limitations of the bincode library. Moving to a universal protocol supported by multiple languages would be the best course of action. Some important definitions below:
What is Bincode?
Bincode is a rust crate for encoding(into bytes) and decoding(into objects) using a tiny binary serialization strategy. One of the selling points for using this library was speed and multi-language support, meaning, other languages can implement the same ahnlich protocol and interact with Ahnlich based on our specifications.

#### Ahnlich Specs: 
This is a JSON document that defines all the types and commands the Ahnlich servers accept per request. This document serves as a contract that ensures correctness between all libraries implemented in other programming languages.


#### Storage Requirements:
Upon startup, the storage configurations for ahnlich can be set and depending on the size can affect the amount of information it can hold. To view the remaining space left, issue the infoserver command, which returns the remaining and the set limit. Ahnlich error once the server reaches or exceeds that limit.

#### Query:
When a message is sent to the ahnlich server the following is checked:
1. #### Message Format: 

    The protocol
    ```
    MAGIC_BYTES(8 bytes) + VERSION_LENGTH(5 bytes) + LENGTH_HEADER_SIZE (8 bytes ) + PAYLOAD_DATA(length of header size)
    ```
    ---

    When a message is received by the ahnlich server, it is read in chunks. 
    `Magic bytes (8 bytes)` represents a keyword (`Ahnlich;`) showing that the message should be processed by the service.

    `Version` represents the semver version of the protocol. If the both parties' `major` version differs, the ahnlich server rejects the connection.

    `Length of Header Size` represents the actual size of the payload sent to be processed. The server needs to know this ahead of time to prepare an internal buffer.

    `Payload Data` represents the request itself. By default, If the size of payload is greater than `1MB`, the ahnlich server errors and drops the message. This Size is configurable though using flags when starting up the server.


2. #### Query Processing:

    Queries in ahnlich are processed sequentially. Meaning in a list of queries, subsequent queries can affect the currently processed query’s output. Example sending a pipelined request(group of requests to be executed sequentially) where the first request drops a store and the next request sets data into that same store. It is paramount to add requests to the queue in the order in which you want them to be processed.


    #### BREAKDOWN OF THE AHNLICH SERVERS

    The ahnlich suite has two servers, each independent of each other. We have a DB and an AI proxy. The primary use of the DB is to store Embeddings and perform similarity retrievals while the AI proxy helps interface with AI models to generate embeddings from text, and Images.


    Ähnlich AI Request/Query Types:

    For more information on the store methods visit this 
    [documentation](https://github.com/deven96/ahnlich/blob/main/docs/ai.md)


    - CreateStore:
    - Get Pred
    - GetSimN
    - CreatePredIndex
    - CreateNonLinearAlgorithmIndex
    - DropPredIndex
    - DropNonLinearAlgorithmIndex
    - Set
    - DelKey
    - DropStore
    - GetKey


    Ahnlich DB Request/Query Types:

    For more information, visit the [db command docs](https://github.com/deven96/ahnlich/blob/main/docs/draft.md#database-server).
    - CreateStore
    - GetKey
    - GetPred
    - GetSimN
    - CreatePredIndex
    - CreateNonLinearAlgorithmIndex
    - DropPredIndex
    - DropNonLinearAlgorithmIndex
    - Set
    - DelKey
    - DelPred
    - DropStore


    Common Requests/Query Types:

    - `InfoServer`: Returns information about the server. You get information like: Tcp address the server is listening on, version, the type of server(AI or DB), the remaining storage left.

    - `ListClients`: Shows Information of all the connected clients
    
    - `ListStores`: Returns a list of stores on the server
    
    - `PurgeStore`: Deletes all the stores on the server
    
    - `Ping`: A health check that shows the service is up and running. Returns PONG


3. #### Persistence:

    The ahnlich server can be configured upon startup to backup the data storage in memory into a persistence location. Currently Ahnlich stores data in a file and the intervals for such operations can be configured at startup also.

    Now that you have some context about Ahnlich


## Proposal
The idea is to swap out our underlying protocol for a language agnostic protocol where any language can be used to easily implement an ahnlich client. GPRC is a strong winner for our protocol because of the following:
- Support and Compatibility: GRPC is a language agnostic protocol that has support in most languages.  So once we come up with our spec in protobuf format, it will be implemented by any language that supports GRPC
- Connection Pooling: GRPC by default supports multiplexing, the clients wouldn’t need to implement their own form of connection pooling thereby reducing the complexity.


## Implementation
Using Grpc as Ahnlich’s underlying means of communication requires: Swapping out both the server's communication and client medium.

First let’s talk about the RPC lifecycle to be employed by Ahnlich. A unary RPC would suffice for the Ahnlich client and server. Where a client sends a single request and gets back a single response.

Further plans might include using a Bidirectional RPC model where both the client and server can stream messages to each other and since there are two independent streams. But for our use case since order is required, the server can wait for all the client messages before writing its messages.

Deadline Or Timeouts: It’s unclear at the moment if we need to add timeouts or deadlines to a particular request or maybe this could be client specific as in the current implementation, there’s a socket timeout when reading a response from the server.


### Proposed Protobuf Implementation of Ahnlich Types:


#### DB Server Query:

- Create Store

    current query
    ```rust

        CreateStore {
            store: StoreName,
            dimension: NonZeroUsize,
            create_predicates: HashSet<MetadataKey>,
            non_linear_indices: HashSet<NonLinearAlgorithm>,
            error_if_exists: bool,
        }
    ```
    protobuf equivalent

    ```protobuf
    message CreateStore {
        string  store = 1;
        uint32  dimension = 2;
        // Since there’s no direct support for hashsets maybe validation would happen when after being parsed
        repeated string create_predicates: 3;
        repeated non_linear_algorthm: 4;
        bool error_if_exists = 5;


    }

    ```
- Get Key

    current

    ```rust

    GetKey {
        store: StoreName,
        keys: Vec<StoreKey>,
    }

    ```
    protobuf
    ```protobuf

    StoreKeys {
        repeated float vectors = 1;
    }

    message GetKey {
        string store = 1;
        repeated StoreKeys keys = 2;
    }
    ```


- Get Predicate

    current 
    ```rust
    GetPred {
            store: StoreName,
            condition: PredicateCondition,
        }
    ```
- Get Sim N

    current 
    ```rust

    GetSimN {
            store: StoreName,
            search_input: StoreKey,
            closest_n: NonZeroUsize,
            algorithm: Algorithm,
            condition: Option<PredicateCondition>,
        }
    ```
- Create Predicate Index

    current
    ```rust
    CreatePredIndex {
            store: StoreName,
            predicates: HashSet<MetadataKey>,
        }
    ```
- Create NonLinear AlgorithmIndex

    current
    ```rust
        CreateNonLinearAlgorithmIndex {
            store: StoreName,
            non_linear_indices: HashSet<NonLinearAlgorithm>,
        }

    ```

- Drop Predicate Index

    current
    ```rust
        DropPredIndex {
            store: StoreName,
            predicates: HashSet<MetadataKey>,
            error_if_not_exists: bool,
        }
    ```

- Drop NonLinear AlgorithmIndex

    current
    ```rust
        DropNonLinearAlgorithmIndex {
            store: StoreName,
            non_linear_indices: HashSet<NonLinearAlgorithm>,
            error_if_not_exists: bool,
        }
    ```

- Set in store

    current
    ```rust
    Set {
            store: StoreName,
            inputs: Vec<(StoreKey, StoreValue)>,
        }
    ```
- Delete Key

    current
    ```rust
    DelKey {
            store: StoreName,
            keys: Vec<StoreKey>,
        }
    ```

- Delete Predicate

    current 
    ```rust
    DelPred {
            store: StoreName,
            condition: PredicateCondition,
        }

    ```
- Drop Store
    
    current
    
    ```rust
    DropStore {
            store: StoreName,
            error_if_not_exists: bool,
        }
    ```
