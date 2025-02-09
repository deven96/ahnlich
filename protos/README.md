# Ahnlich Protofiles

This folder contains Protocol Buffers (protobuf) definitions used by Ahnlich. These definitions establish structured communication between ahnlich services.

## **Overview**
The proto files define key components such as:

- **Metadata:** Defines structured data values.
- **Query System:** Allows querying with predicates and filtering.
- **Similarity Search:** Returns Most similar store entries based on different algorithms.
- **Algorithms:** Supported algorthims (linear and nonlinear). 
- **AI Models & Execution Providers:** Defines AI inference settings.

## **Protofile Descriptions**
### **Metadata (`metadata.proto`)**
Defines metadata types and values.
- `MetadataType`: Enum representing types of metadata (e.g., `RAW_STRING`, `IMAGE`).
- `MetadataValue`: A message holding metadata in either string or byte format.

### **Predicates (`predicate.proto`)**
Defines query predicates used to filter stored data.
- `Predicate`: Supports conditions like `Equals`, `NotEquals`, `In`, and `NotIn`.
- `PredicateCondition`: Allows complex conditions using `AND` and `OR`.

### **Key-Value (`keyval.proto`)**
Defines structured storage keys and values.
- `StoreKey`: Represents keys using floating point arrays.
- `StoreValue`: Stores metadata in a key-value format.

### **Database Queries (`db/query.proto`)**
Defines database operations such as:
- Creating stores (`CreateStore`).
- Fetching keys (`GetKey`).
- Filtering (`GetPred`).
- Performing similarity searches (`GetSimN`).

### **Similarity (`similarity.proto`)**
Defines similarity scoring.
- `Similarity`: A float value representing similarity between stored and queried data.

### **Server Communication (`server_types.proto`, `client.proto`)**
Defines server types and client connections.
- `ServerType`: Enum for AI or Database services.
- `ConnectedClient`: Holds client address and connection time.

### **AI & Algorithm Definitions**
- `algorithm.proto`: Defines similarity algorithms (e.g., `CosineSimilarity`).
- `ai/models.proto`: Defines AI models available for use.
- `ai/execution_provider.proto`: Specifies execution providers (e.g., `CUDA`, `TENSOR_RT`).

## **Usage**
To use these protofiles, generate language-specific gRPC stubs:

```sh
protoc --proto_path=. --go_out=./gen --go-grpc_out=./gen *.proto
```


### Rust: Automatic Generation

In `grpc_types`, cargo automatically compiles the .proto files via the `tonic` and `prost` crate,  generating the rust equivalent.



### Python: Generation via pyproject.toml

For Python, we use `betterproto` to generate types automatically.
