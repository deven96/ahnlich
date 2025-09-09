---
title: Info Server
---

# Info Server

How to request server information from the Ahnlich AI Service using the Python client.

AI Requests are the fundamental way to interact with the Ahnlich AI backend. They provide low-level functionality such as checking availability, inspecting runtime configuration, and requesting embeddings or search results.

In the Ahnlich Python client programming model, AI requests are made through stubs generated from the gRPC definitions. Each request must be wrapped in an `async` function, as communication is asynchronous.

The following example demonstrates how to make an Info Server request.

<details>
  <summary>Click to expand source code</summary>

  ```py
  import asyncio
  from grpclib.client import Channel
  from ahnlich_client_py.grpc.services.ai_service import AiServiceStub
  from ahnlich_client_py.grpc.ai import query as ai_query


  async def info_server():
    async with Channel(host="127.0.0.1", port=1370) as channel:
        client = AiServiceStub(channel)
        response = await client.info_server(ai_query.InfoServer())
        print(response) #InfoServer(info=ServerInfo(address='Ok(0.0.0.0:1370)', version='0.1.0', limit=10073741824, remaining=10067931251))


  if __name__ == "__main__":
    asyncio.run(info_server())
  ```
</details>

## Define Request Parameters

The `InfoServer` request does not require any parameters.
It is used primarily for **diagnostics and service discovery**, returning metadata such as:

* Server version

* Supported models

* Available features

* Runtime configuration details

## Define Response Handling

The response from `info_server()` is serializable and can be logged or stored.
It should be treated as **read-only diagnostic data** and is most commonly used when:

* Debugging compatibility issues

* Verifying that the AI service is initialized

* Inspecting model availability before embedding or search requests

