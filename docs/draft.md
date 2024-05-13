## Mascot

## Overall Goals

### Must haves
- Ability to store, modify, and retrieve vector string key value pairs
- Ability to retrieve vector string key pairs by algorithmic similarity:

    This means having to the ability to use various types of algorithms to compare existing vector string key pairs
- Well-defined protocol spec to enable easy building of libraries in multiple languages

- Ability to create multiple stores within the db
- CLI for communicating with the database server

- Scalability of internal datastructures:
    example for places where we want to use a hashmap we'd use a concurrency hashmap like [flurry](https://github.com/jonhoo/flurry) by jonhoo or we implement ours ourselves using read-write locks. (Check the internal workings of flurry. In what cases do writes block readers. THat will form the basis in how the system behaves)

- Server should properly handle SIG terms: 
    It should properly cleanup resources and save indices to disk, etc

### Nice to haves
- Persistence: 
    The ability to optimally store and recover records on disk. This would involve looking into various compaction methods and what works best for us.

    <b> Can be enabled/disabled behind a flag</b>

- Predicate functions: This would mean we enforce that our values are json objects with key and values themselves. Subsequently, we can index in the value objects to reduce the search space using the predicate.

    - Internal indices to help speed up predicates?
    - Decide if predicates are combineable. What functions do we want to support for predicates(And -> Intersection across predicate indices, OR -> Union)
    - Predicate search should not also be equality but also inequality
    - Decide the syntax to send syntax across the wire 
    - The ability to decided what to index:
    Server should properly handle indices


- Client Library retry mechanisms with respect to network errors or partitions.

## NOTE:
** Look out for Throughput of the system(messages processed/ second) with respect to async/sync



## Architecture Diagram

- TODO: Ensure to draw the diagram to include the components of the system

## Components of the system
### Database server
- Internal data structure:
    We need to come up with optimal ways to set, get and search by similarity using various algorithms. This would mean we support various algorthims, we'd start with the simple ones like:
    
    - Euclidean similarity
    - cosine similarity
    - dot-product
    
    Internal data structures should also plan for the asynchronous update of predicate indices

    With respect to arrays, we're gonna use the ndarray, and we're always gonna assume that the y axis is dimension of 1
- storage and persistence handlers(<b>--flag</b>)

    The ability to store and retrieve data structure representations from disk.
    Look into various optimal representation on disk(storage and retrival)

- Custom TCP protocol

    This represents all the actions that can be perform by the server and any connecting client across the wire.

    This will entail coming up with a version spec.
    Ensure same version spec on handshake between client and server

    Check if we can serialize and deserialize across the wire(bincode/serde). The consideration as to which to use might include speed(bincode wins) or consistency across multiple languages. We can also consider MessagePack which is smaller and simpler to parse than JSON but is also easy to debug


    Here's a rough sketch of commands to be expanded on later:
    
    - `CONNECT`
    - `DISCONNECT`
    - `SHUTDOWNSERVER`: shut down basically discounts from all connected clients, performs cleanup before killing the server
    - `CREATE`: Create a store which must have a unique name with respect to the server.
    Create can take in name_of_store, dimensions_of_vectors(immutable) to be stored in that store, ability to create predicate indices
    - `GETKEY`: takes in store, key and direct return of key within store matching the input key

    - `GETPRED`:takes in store, and predicates and returns all values matching predicates
    
        Validation should check if predicate was enabled else error.

        returns 0 if no predicate was found in json value


    - `GETSIMN`:get similar n takes in store and supported algorithm, a reference input vector, predicate,
    n which is the max number of returns it will return

        Underneath this uses a min or max heap depending on the algorithm to compare most similar vector. 
        
        This becomes Linear in the end
    validation is done based on the input vector dimensions and the store's dimensions. They must match.

    Validation should check if predicate was enabled.

    - `REINDEX`: Reindexes a store using some new predicates. Adds the predicates that did not exist and does not remove existing predicates
    - `DROPINDEXPRED`: takes in predicate, store and drops the predicate for that store

    - `SET`: takes in store, length of input vector, input of which each should have a key matching store dimension and value of type json.

        
        Validation is on all individual vectors before any update operation is performed

        Should be idempotent
        
        We will assume inputs to be a hashmap

        update indices as much as possible without blocking the return of response
    - `DELKEY`: Takes in a store, and key and performs a delete matching that key.
    returns number of keys deleted. It should also update indices in a non-blocking way

    - `DELPRED`: Takes in a store, key and delete all values matching that predicate. It should also update indices in a non-blocking way.

        Validation should check if predicate was enabled.
    - `DROPSTORE`: takes in a store and deletes it. Destroys everything pertaining the store

    - `INFOSERVER`: returns the server information such as port, host, version, etc.

    - `LISTSTORES`: List all the stores on the server. It also returns information like store length/size.

    - `LISTCLIENTS`: Returns a list of clients connected to the server

    ### Clients

    #### Language Clients

    We would be implementing for starters a rust and python client for simplicity sake, the client always ensures that it matchings the version of the server upon connecting.

    Potential retry capabilites for the clients whenever network errors are returned by the server.

    Look into Type generation to ensure we don't have to duplicate across different languages, so adding a new language is simple as writing boilerplates

    Ensure to build appropriate docs and reference them in the readme.

    #### CLI Clients

    An Ahnlich CLI binary should be downloadable. This can wrap around the rust library taking string inputs.
    Should be able to parse string to the needed data structure


## Drawbacks of the system
- In-memory: with the potential of disk storage that increases latency
** Measure Throughput for important commands before we highlight pain points

- Memory consumption

TODO:
- any other drawbacks we find as we move 



## Testing and Fuzzing
Every component of the system should be testable

All libriaries should be testable with a running version of the server, same with the cli


## Tracing Logs and Profiling

The server should give the ability to connect and otlp endpoint in which to send traces to.

Tracing should be activatable.

Logs should be activatable.

## Deployment and CI/CD
All libraries should be deployed to their appropriate repositories while binaries should be signed and deployed on github releases by the CI


## Version changes

When major version matches, clients and servers should be able to communicate.

Version changes should match branches on github that we use for release.


## Publications
- Always update readme to reflect current state of the project.

- Write a couple of articles and guides/wikis on how to use.
- We can have a spec in a different documents so others can implement libs in various languages. Find a way to extract rust types and signature into markdown

- Attend one or two talks

- An AI microservice(small python service) should use Ahnlich DB
