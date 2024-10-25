# Ahnlich
<p align="left"><img src="assets/logo.jpg" alt="ahnlich" height="120px"></p>

[![All Test](https://github.com/deven96/ahnlich/actions/workflows/test.yml/badge.svg)](https://github.com/deven96/ahnlich/actions/workflows/test.yml)

ähnlich means similar in german. It comprises of multiple tools for usage and development such as:

- [`ahnlich-db`](ahnlich/db): In-memory vector key value store for storing embeddings/vectors with corresponding metadata(key-value maps). It's a powerful system which enables AI/ML engineers to store and search similar vectors using linear (cosine, euclidean) or non-linear similarity (kdtree) algorithms. It also leverages search within metadata values to be able to filter out entries using metadata values. A simple example can look like
```
GETSIMN 2 WITH [0.2, 0.1] USING cosinesimilarity IN my_store WHERE (page != hidden)

// example query
get_sim_n(
    store="my_store",
    search_input=[0.2, 0.1],
    closest_n=2,
    algorithm=CosineSimilarity,
    condition=Predicate::NotEquals{
      key="page",
      value="hidden",
  },
)
```
- [`ahnlich-ai`](ahnlich/ai/): AI proxy to communicate with `ahnlich-db`, receiving raw input, transforming into embeddings, and storing within the DB. It extends the capabilities by then allowing developers/engineers to issue queries to the same store using raw input such as images/text. It features multiple off-the-shelf models that can be selected for store index and query.
```
CREATESTORE my_store QUERYMODEL all-minilm-l6-v2 INDEXMODEL all-minilm-l6-v2

// example query
create_store(
    store="my_store",
    index_model="all-minilm-l6-v2",
    query_model="all-minilm-l6-v2",
)
```
- [`ahnlich-client-rs`](ahnlich/client/): Rust client for `ahnlich-db` and `ahnlich-ai` with support for connection pooling.
- [`ahnlich-client-py`](sdk/ahnlich-client-py/): Python client for `ahnlich-db` and `ahnlich-ai` with support for connection pooling.

- [`ahnlich-cli`](ahnlich/cli/): CLI for querying `ahnlich-db` and `ahnlich-ai`


## Architecture

![Architecture Diagram](assets/ahnlich.jpg)


## Usage

`ahnlich-db`, `ahnlich-ai` and `ahnlich-cli` are packaged and released as [binaries](https://github.com/deven96/ahnlich/releases) for multiple platforms alongside [docker images](https://github.com/deven96?tab=packages&repo_name=ahnlich)

The DB can be used without the AI proxy for more fine grained control of the generated vector embeddings as all clients support both

### Docker Images.

`Note`: 
1. Arguments and commands must be passed in quotes. E.G: `docker run <image_name> "ahnlich-db run --enable-tracing --port 8000"`

2. The CLI comes packaged into the docker images.



## Development

### Using Spec documents to interact with Ahnlich DB

To generate the spec documents, run
```bash
cargo run --bin typegen generate
```
It is worth noting that any changes to the types crate, requires you to run the above command. This helps keep our spec document and types crate in sync.

To Convert spec documents to a programming language, run:

```bash
 cargo run --bin typegen create-client <Programming Language>
```
Available languages are:
- python
- golang
- typescript.

In order to communicate effectively with the ahnlich db, you would have to extend the bincode serialization protocol automatically provided by `serde_generate`.
Your message(in bytes) should be serialized and deserialized in the following format => `AHNLICH_HEADERS` + `VERSION` + `QUERY/SERVER_RESPONSE`. Bytes are `Little Endian`.


### How Client Releases Work

The clients follow a similar process when deploying new releases.
[Example with python client](https://github.com/deven96/ahnlich/blob/main/sdk/ahnlich-client-py/README.md#deploy-to-artifactory).





