---
title: Info Server
sidebar_posiiton: 2
---

# Info Server

The Info Server request retrieves metadata about the running DB server, including the binary version and server type (DB, AI, or Hybrid). This is useful for environment validation, feature gating, and diagnostics.

## Behavior

* The client sends an InfoServer request.

* The server responds with version and type metadata.

* Clients can use this to validate they are connected to the correct server role.

<details>
  <summary>Click to expand source code</summary>

```py
import asyncio
from grpclib.client import Channel
from ahnlich_client_py.grpc.services.db_service import DbServiceStub
from ahnlich_client_py.grpc.db import query as db_query
from ahnlich_client_py.grpc.server_types import ServerType

async def info_server():
  """Test server version"""

  async with Channel(host="127.0.0.1", port=1369) as channel:
    client = DbServiceStub(channel)
    response = await client.info_server(db_query.InfoServer())
    # response contains server version and type
    print(f"Server version: {response.info.version}")

if __name__ == "__main__":
  asyncio.run(info_server())
```
</details>