## Ahnlich Client PY

A Python client that interacts with both ahnlich DB and AI


[![Ahnlich TestSuite](https://github.com/deven96/ahnlich/actions/workflows/test.yml/badge.svg)](https://github.com/deven96/ahnlich/actions/workflows/test.yml)
[![Ahnlich Python Client Tag and Deploy](https://github.com/deven96/ahnlich/actions/workflows/python_tag_and_deploy.yml/badge.svg)](https://github.com/deven96/ahnlich/actions/workflows/python_tag_and_deploy.yml)

## Usage Overview

The following topics are covered:
* [Installation](#installation)
* [Package Information](#package-information)
* [Server Response](#server-response)
* [Initialization](#initialization)
    * [Client](#client)

* [Connection Pooling](#connection-pooling)
* [Requests - DB](#requests---db)
    * [Ping](#ping)
    * [Info Server](#info-server)
    * [List Connected Clients](#list-connected-clients)
    * [List Stores](#list-stores)
    * [Create Store](#create-store)
    * [Set](#set)
    * [Drop Store](#drop-store)
    * [Get Sim N](#get-sim-n)
    * [Get Key](#get-key)
    * [Get By Predicate](#get-by-predicate)
    * [Create Predicate Index](#create-predicate-index)
    * [Drop Predicate Index](#drop-predicate-index)
    * [Create Non Linear Algorithm Index](#create-non-linear-algorithm-index)
    * [Drop Non Linear Algorithm Index](#drop-non-linear-algorithm-index)
    * [Delete Key](#delete-key)
    * [Delete Predicate](#delete-predicate)

* [Requests - AI](#requests---ai)
    * [Ping](#ping-1)
    * [Info Server](#info-server-1)
    * [List Stores](#list-stores-1)
    * [Create Store](#create-store-1)
    * [Set](#set-1)
    * [Drop Store](#drop-store-1)
    * [Get Sim N](#get-sim-n-1)
    * [Get By Predicate](#get-by-predicate-1)
    * [Create Predicate Index](#create-predicate-index-1)
    * [Drop Predicate Index](#drop-predicate-index-1)
    * [Create Non Linear Algorithm Index](#create-non-linear-algorithm-index-1)
    * [Drop Non Linear Algorithm Index](#drop-non-linear-algorithm-index-1)
    * [Delete Key](#delete-key-1)

* [Bulk Requests](#bulk-requests)
* [Client As Context Manager](#client-as-context-manager)
* [How to Deploy to Artifactory](#deploy-to-artifactory)
* [Type Meanings](#type-meanings)
* [Change Log](#change-log)

## Installation

- Using Poetry
```bash
poetry add ahnlich-client-py
```
- Using pip
```bash
pip3 install ahnlich-client-py
```

## Package Information
The ahnlich client has some noteworthy modules that should provide some context
- grpclib


## Server Response

All db query types have an associating server response, all which can be found
```py
from ahnlich_client_py.grpc.db import server
```
For AI Server
```py
from ahnlich_client_py.grpc.ai import server
```

## Initialization

### Client

```py
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import db_service


channel = Channel(host="127.0.0.1", port=1369)
    client = db_service.DbServiceStub(channel)
```


## Requests - DB

### Ping

```py

from grpclib.client import Channel
from ahnlich_client_py.grpc.services import DbServiceStub
from ahnlich_client_py.grpc.db import query as db_query

# Initialize client
async with Channel(host="127.0.0.1", port=1369) as channel:
    db_client = DbServiceStub(channel)
    
    # Prepare tracing metadata
    tracing_id = "00-80e1afed08e019fc1110464cfa66635c-7a085853722dc6d2-01"
    metadata = {"ahnlich-trace-id": tracing_id}
    
    # Make request with metadata
    response = await db_client.ping(
        db_query.Ping(),
        metadata=metadata
    )
    
    print(response)  # Returns Pong message
```

###  Info Server

```py
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import DbServiceStub
from ahnlich_client_py.grpc.db import query as db_query

async with Channel(host="127.0.0.1", port=1369) as channel:
    client = DbServiceStub(channel)
    response = await client.info_server(db_query.InfoServer())
    # response contains server version and type
    print(f"Server version: {response.info.version}")

```

###  List Connected Clients 

```py
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import DbServiceStub
from ahnlich_client_py.grpc.db import query as db_query

async with Channel(host="127.0.0.1", port=1369) as channel:
    client = DbServiceStub(channel)
    response = await client.list_clients(db_query.ListClients())
    print(f"Connected clients: {[c.id for c in response.clients]}")
```

###  List Stores

```py
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import DbServiceStub
from ahnlich_client_py.grpc.db import query as db_query

async with Channel(host="127.0.0.1", port=1369) as channel:
    client = DbServiceStub(channel)
    tracing_id = "00-80e1afed08e019fc1110464cfa66635c-7a085853722dc6d2-01"
    response = await client.list_stores(
        db_query.ListStores(),
        metadata={"ahnlich-trace-id": tracing_id}
    )
    print(f"Stores: {[store.name for store in response.stores]}")
```

###  Create Store

```py
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import DbServiceStub
from ahnlich_client_py.grpc.db import query as db_query

async with Channel(host="127.0.0.1", port=1369) as channel:
    client = DbServiceStub(channel)
    response = await client.create_store(
        db_query.CreateStore(
            store="test store",
            dimension=5,  # Fixed vector dimension
            predicates=["job"],  # Index these metadata fields
            error_if_exists=True
        )
    )
    # response is Unit() on success
    
    # All store_keys must match this dimension
    # Example valid key:
    valid_key = [1.0, 2.0, 3.0, 4.0, 5.0]  # length = 5
```
Once store dimension is fixed, all `store_keys` must confirm with said dimension.
Note we only accept 1 dimensional arrays/vectors of length N.
Store dimensions is a one dimensional array of length N


### Set
```py
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import DbServiceStub
from ahnlich_client_py.grpc import keyval, metadata
from ahnlich_client_py.grpc.db import query as db_query

async with Channel(host="127.0.0.1", port=1369) as channel:
    client = DbServiceStub(channel)
    
    store_key = keyval.StoreKey(key=[5.0, 3.0, 4.0, 3.9, 4.9])
    store_value = keyval.StoreValue(
        value={"rank": metadata.MetadataValue(raw_string="chunin")}
    )
    
    response = await client.set(
        db_query.Set(
            store="test store",
            inputs=[keyval.DbStoreEntry(key=store_key, value=store_value)]
        )
    )
    # response contains upsert counts (inserted, updated)
```


### Drop store
```py
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import DbServiceStub
from ahnlich_client_py.grpc.db import query as db_query

async with Channel(host="127.0.0.1", port=1369) as channel:
    client = DbServiceStub(channel)
    
    response = await client.drop_store(
        db_query.DropStore(
            store="test store",
            error_if_not_exists=True
        )
    )
    # response contains deleted_count
```


### Get Sim N
Returns an array of tuple of (store_key, store_value) of Maximum specified N

```py
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import DbServiceStub
from ahnlich_client_py.grpc.db import query as db_query
from ahnlich_client_py.grpc.shared.algorithm import Algorithm

async with Channel(host="127.0.0.1", port=1369) as channel:
    client = DbServiceStub(channel)
    
    search_key = keyval.StoreKey(key=[...])  # Your query vector
    
    response = await client.get_sim_n(
        db_query.GetSimN(
            store="test store",
            search_input=search_key,
            closest_n=3,  # Must be > 0
            algorithm=Algorithm.CosineSimilarity
        )
    )
    # response.entries contains (key, value, similarity) tuples
```
<u>*Closest_n is a Nonzero integer value*</u>



### Get Key
Returns an array of tuple of (store_key, store_value)

```py
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import DbServiceStub
from ahnlich_client_py.grpc.db import query as db_query

async with Channel(host="127.0.0.1", port=1369) as channel:
    client = DbServiceStub(channel)
    
    lookup_key = keyval.StoreKey(key=[...])  # Your lookup vector
    
    response = await client.get_key(
        db_query.GetKey(
            store="test store",
            keys=[lookup_key]
        )
    )
    # response.entries contains matching (key, value) pairs
```


### Get By Predicate
Same as Get_key but returns results based defined conditions

```py
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import DbServiceStub
from ahnlich_client_py.grpc.db import query as db_query
from ahnlich_client_py.grpc import predicates, metadata

async with Channel(host="127.0.0.1", port=1369) as channel:
    client = DbServiceStub(channel)
    
    condition = predicates.PredicateCondition(
        value=predicates.Predicate(
            equals=predicates.Equals(
                key="job",
                value=metadata.MetadataValue(raw_string="sorcerer")
            )
        )
    )
    
    response = await client.get_pred(
        db_query.GetPred(
            store="test store",
            condition=condition
        )
    )
    # response.entries contains matching items
```

### Create Predicate Index
```py
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import DbServiceStub
from ahnlich_client_py.grpc.db import query as db_query

async with Channel(host="127.0.0.1", port=1369) as channel:
    client = DbServiceStub(channel)
    
    response = await client.create_pred_index(
        db_query.CreatePredIndex(
            store="test store",
            predicates=["job", "rank"]
        )
    )
    # response.created_indexes shows how many indexes were created
```

### Drop Predicate Index
```py
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import DbServiceStub
from ahnlich_client_py.grpc.db import query as db_query

async with Channel(host="127.0.0.1", port=1369) as channel:
    client = DbServiceStub(channel)
    
    response = await client.drop_pred_index(
        db_query.DropPredIndex(
            store="test store",
            predicates=["job"],
            error_if_not_exists=True
        )
    )
    # response.deleted_count shows how many indexes were removed
```

### Create Non Linear Algorithm Index
```py
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import DbServiceStub
from ahnlich_client_py.grpc.db import query as db_query
from ahnlich_client_py.grpc.algorithm.algorithms import NonLinearAlgorithm

async with Channel(host="127.0.0.1", port=1369) as channel:
    client = DbServiceStub(channel)
    
    response = await client.create_non_linear_algorithm_index(
        db_query.CreateNonLinearAlgorithmIndex(
            store="test store",
            non_linear_indices=[NonLinearAlgorithm.KDTree]
        )
    )
    # response.created_indexes shows how many indexes were created
```

### Drop Non Linear Algorithm Index
```py
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import DbServiceStub
from ahnlich_client_py.grpc.db import query as db_query
from ahnlich_client_py.grpc.algorithm.algorithms import NonLinearAlgorithm

async with Channel(host="127.0.0.1", port=1369) as channel:
    client = DbServiceStub(channel)
    
    response = await client.drop_non_linear_algorithm_index(
        db_query.DropNonLinearAlgorithmIndex(
            store="test store",
            non_linear_indices=[NonLinearAlgorithm.KDTree],
            error_if_not_exists=True
        )
    )
    # response.deleted_count shows how many indexes were removed
```


### Delete Key
```py
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import DbServiceStub
from ahnlich_client_py.grpc.db import query as db_query
from ahnlich_client_py.grpc import keyval

async with Channel(host="127.0.0.1", port=1369) as channel:
    client = DbServiceStub(channel)
    
    store_key = keyval.StoreKey(key=[5.0, 3.0, 4.0, 3.9, 4.9])
    
    response = await client.del_key(
        db_query.DelKey(
            store="test store",
            keys=[store_key]
        )
    )
    # response.deleted_count shows how many items were deleted
```

### Delete Predicate
```py
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import DbServiceStub
from ahnlich_client_py.grpc.db import query as db_query
from ahnlich_client_py.grpc import predicates, metadata

async with Channel(host="127.0.0.1", port=1369) as channel:
    client = DbServiceStub(channel)
    
    condition = predicates.PredicateCondition(
        value=predicates.Predicate(
            equals=predicates.Equals(
                key="job",
                value=metadata.MetadataValue(raw_string="sorcerer")
            )
        )
    )
    
    response = await client.del_pred(
        db_query.DelPred(
            store="test store",
            condition=condition
        )
    )
    # response.deleted_count shows how many items were deleted
```


## Requests - AI


### Ping

```py
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import AiServiceStub
from ahnlich_client_py.grpc.ai import query as ai_query

async with Channel(host="127.0.0.1", port=1370) as channel:
    client = AiServiceStub(channel)
    response = await client.ping(ai_query.Ping())
```

###  Info Server

```py
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import AiServiceStub
from ahnlich_client_py.grpc.ai import query as ai_query

async with Channel(host="127.0.0.1", port=1370) as channel:
    client = AiServiceStub(channel)
    response = await client.info_server(ai_query.InfoServer())
```

###  List Stores

```py
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import AiServiceStub
from ahnlich_client_py.grpc.ai import query as ai_query

async with Channel(host="127.0.0.1", port=1370) as channel:
    client = AiServiceStub(channel)
    response = await client.list_stores(ai_query.ListStores())

```

###  Create Store

```py

from grpclib.client import Channel
from ahnlich_client_py.grpc.services import AiServiceStub
from ahnlich_client_py.grpc.ai import query as ai_query
from ahnlich_client_py.grpc.ai.models import AiModel

async with Channel(host="127.0.0.1", port=1370) as channel:
    client = AiServiceStub(channel)
    response = await client.create_store(
        ai_query.CreateStore(
            store="test store",
            query_model=AiModel.ALL_MINI_LM_L6_V2,
            index_model=AiModel.ALL_MINI_LM_L6_V2,
            predicates=["job"],
            error_if_exists=True,
            # Store original controls if we choose to store the raw inputs 
            # within the DB in order to be able to retrieve the originals again
            # during query, else only store values are returned
            store_original=True
        )
    )

```


### Set
```py

from grpclib.client import Channel
from ahnlich_client_py.grpc.services import AiServiceStub
from ahnlich_client_py.grpc.ai import query as ai_query
from ahnlich_client_py.grpc import keyval, metadata
from ahnlich_client_py.grpc.ai import preprocess

async with Channel(host="127.0.0.1", port=1370) as channel:
    client = AiServiceStub(channel)
    response = await client.set(
        ai_query.Set(
            store="test store",
            inputs=[
                keyval.AiStoreEntry(
                    key=keyval.StoreInput(raw_string="Jordan One"),
                    value=keyval.StoreValue(
                        value={"brand": metadata.MetadataValue(raw_string="Nike")}
                    ),
                ),
                keyval.AiStoreEntry(
                    key=keyval.StoreInput(raw_string="Yeezey"),
                    value=keyval.StoreValue(
                        value={"brand": metadata.MetadataValue(raw_string="Adidas")}
                    ),
                )
            ],
            preprocess_action=preprocess.PreprocessAction.NoPreprocessing
        )
    )
```


### Drop store
```py
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import AiServiceStub
from ahnlich_client_py.grpc.ai import query as ai_query

async with Channel(host="127.0.0.1", port=1370) as channel:
    client = AiServiceStub(channel)
    response = await client.drop_store(
        ai_query.DropStore(
            store="test store",
            error_if_not_exists=True
        )
    )


```


### Get Sim N ??
Returns an array of tuple of (store_key, store_value) of Maximum specified N

```py
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import AiServiceStub
from ahnlich_client_py.grpc.ai import query as ai_query
from ahnlich_client_py.grpc import keyval, algorithms

async with Channel(host="127.0.0.1", port=1370) as channel:
    client = AiServiceStub(channel)
    
    response = await client.get_sim_n(
        ai_query.GetSimN(
            store="test store",
            search_input=keyval.StoreInput(raw_string="Jordan"),
            closest_n=3,
            algorithm=algorithms.Algorithm.COSINE_SIMILARITY,
            condition=None,  # Optional predicate condition
            execution_provider=None  # Optional execution provider
        )
    )
    
    # Response contains entries with similarity scores
    for entry in response.entries:
        print(f"Key: {entry.key.raw_string}")
        print(f"Score: {entry.score}")
        print(f"Value: {entry.value}")
```
<u>*Closest_n is a Nonzero integer value*</u>


### Get By Predicate
Same as Get_key but returns results based defined conditions

```py
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import AiServiceStub
from ahnlich_client_py.grpc.ai import query as ai_query
from ahnlich_client_py.grpc import predicates, metadata

async with Channel(host="127.0.0.1", port=1370) as channel:
    client = AiServiceStub(channel)
    condition = predicates.PredicateCondition(
        value=predicates.Predicate(
            equals=predicates.Equals(
                key="brand", 
                value=metadata.MetadataValue(raw_string="Nike")
            )
        )
    )
    response = await client.get_pred(
        ai_query.GetPred(
            store="test store",
            condition=condition
        )
    )
```

### Create Predicate Index
```py
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import AiServiceStub
from ahnlich_client_py.grpc.ai import query as ai_query

async with Channel(host="127.0.0.1", port=1370) as channel:
    client = AiServiceStub(channel)
    response = await client.create_pred_index(
        ai_query.CreatePredIndex(
            store="test store",
            predicates=["job", "rank"]
        )
    )
```

### Drop Predicate Index
```py
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import AiServiceStub
from ahnlich_client_py.grpc.ai import query as ai_query

async with Channel(host="127.0.0.1", port=1370) as channel:
    client = AiServiceStub(channel)
    response = await client.drop_pred_index(
        ai_query.DropPredIndex(
            store="test store",
            predicates=["job"],
            error_if_not_exists=True
        )
    )
```

### Create Non Linear Algorithm Index
```py
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import AiServiceStub
from ahnlich_client_py.grpc.ai import query as ai_query
from ahnlich_client_py.grpc.algorithm.nonlinear import NonLinearAlgorithm

async with Channel(host="127.0.0.1", port=1370) as channel:
    client = AiServiceStub(channel)
    response = await client.create_non_linear_index(
        ai_query.CreateNonLinearIndex(
            store="test store",
            algorithms=[NonLinearAlgorithm.KDTree],
            error_if_exists=True
        )
    )
```

### Drop Non Linear Algorithm Index
```py
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import AiServiceStub
from ahnlich_client_py.grpc.ai import query as ai_query
from ahnlich_client_py.grpc.algorithm.nonlinear import NonLinearAlgorithm

async with Channel(host="127.0.0.1", port=1370) as channel:
    client = AiServiceStub(channel)
    response = await client.drop_non_linear_index(
        ai_query.DropNonLinearIndex(
            store="test store",
            algorithms=[NonLinearAlgorithm.KDTree],
            error_if_not_exists=True
        )
    )
```



### Delete Key
```py

from grpclib.client import Channel
from ahnlich_client_py.grpc.services import AiServiceStub
from ahnlich_client_py.grpc.ai import query as ai_query
from ahnlich_client_py.grpc import keyval

async with Channel(host="127.0.0.1", port=1370) as channel:
    client = AiServiceStub(channel)
    response = await client.del_key(
        ai_query.DelKey(
            store="test store",
            key=keyval.StoreInput(raw_string="Custom Made Jordan 4")
        )
    )
```


#### Tracing ID example:

```py
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import AiServiceStub
from ahnlich_client_py.grpc.ai import query as ai_query

async with Channel(host="127.0.0.1", port=1370) as channel:
    client = AiServiceStub(channel)
    
    # Prepare tracing metadata
    tracing_id = "00-80e1afed08e019fc1110464cfa66635c-7a085853722dc6d2-01"
    metadata = {"ahnlich-trace-id": tracing_id}
    
    # Make request with metadata
    response = await client.ping(
        ai_query.Ping(),
        metadata=metadata
    )
```


## Bulk Requests
Clients have the ability to send multiple requests at once, and these requests will be handled sequentially. The builder class takes care of this. The response is a list of all individual request responses.


```py
from ahnlich_client_py import AhnlichDBClient
client = AhnlichDBClient(address="127.0.0.1", port=port)

request_builder = client.pipeline()
request_builder.ping()
request_builder.info_server()
request_builder.list_clients()
request_builder.list_stores()

response: server_response.ServerResult = client.exec()
```
*Sample applies to the AIclient*


## Deploy to Artifactory

Replace the contents of `MSG_TAG` file with your new tag message

From Feature branch, either use the makefile :
```bash
make bump-py-client BUMP_RULE=[major, minor, patch] 
```
or
```bash
poetry run bumpversion [major, minor, patch] 
```

When Your PR is made, changes in the client version file would trigger a release build to Pypi


## Type Meanings

- Store Key: A one dimensional vector
- Store Value: A Dictionary containing texts or binary associated with a storekey
- Store Predicates: Or Predicate indices are basically indices that improves the filtering of store_values
- Predicates: These are operations that can be used to filter data(Equals, NotEquals, Contains, etc)
- PredicateConditions: They are conditions that utilize one predicate or tie Multiple predicates together using the AND, OR or Value operation. Where Value means just a predicate.
Example: 
Value
```py
condition = predicates.PredicateCondition(
            value=predicates.Predicate(
                equals=predicates.Equals(
                    key="job", value=metadata.MetadataValue(raw_string="sorcerer")
                )
            )
        )
```
Metadatavalue can also be a binary(list of u8s)

```py

condition = predicates.PredicateCondition(
            value=predicates.Predicate(
                equals=predicates.Equals(
                    key="rank", value=metadata.MetadataValue(image=[2,2,3,4,5,6,7])
                )
            )
        )
```


AND 



```py

condition = predicates.PredicateCondition(
    and_=predicates.AndCondition(
        left=predicates.PredicateCondition(
            value=predicates.Predicate(
                equals=predicates.Equals(
                    key="job",
                    value=metadata.MetadataValue(raw_string="sorcerer")
                )
            )
        ),
        right=predicates.PredicateCondition(
            value=predicates.Predicate(
                equals=predicates.Equals(
                    key="rank",
                    value=metadata.MetadataValue(raw_string="chunin")
                )
            )
        )
    )
)

```

- Search Input: A string or binary file that can be stored by the aiproxy. Note, the binary file depends on the supported models used in a store or supported by Ahnlich AI

- AIModels: Supported AI models used by ahnlich ai
- AIStoreType: A type of store to be created. Either a Binary or String

## Change Log

| Version| Description           |
| -------|:-------------:|
| 0.0.0 | Base Python clients (Async and Sync) to connect to ahnlich db and AI, with connection pooling and Bincode serialization and deserialization |
| 1.0.0 | Rewrite Underlying communication using GRPC |



