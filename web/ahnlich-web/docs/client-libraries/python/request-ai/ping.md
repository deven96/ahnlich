---
title: Ping
---

# Ping

The Ping request verifies connectivity with the AI Service.
It works just like the DB Ping, but instead communicates with the AI server (running on port 1370 by default).

* **Input**:
  * No additional fields are required.

* **Behavior**: Sends a lightweight request to check if the AI Service is responsive.

* **Response**:
  * Returns a `Pong` message if the AI service is alive.


<details>
  <summary>Click to expand source code</summary>

  ```py
  import asyncio
  from grpclib.client import Channel
  from ahnlich_client_py.grpc.services.ai_service import AiServiceStub
  from ahnlich_client_py.grpc.ai import query as ai_query


  async def ping():
    async with Channel(host="127.0.0.1", port=1370) as channel:
        client = AiServiceStub(channel)
        response = await client.ping(ai_query.Ping())
    print(response) #Pong()


  if __name__ == "__main__":
    asyncio.run(ping())
  ```
</details>