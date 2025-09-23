---
title: Python Specific Resources
sidebar_posiiton: 1
---

## QuickStart

Set up your local development environment to start building with Ahnlich

## Install Python

Go to the official [Python website](https://www.python.org/downloads/)

Ensure that Python is installed on your system, and verify its version by running the following command in your terminal:

```
  python3 -V
```

*Example output:*

```
  Python 3.13.3
```

## Set Up a Local Python Environment

Use a Python virtual environment to ensure your Ahnlich client python sdk  runs in an isolated and consistent setup using the command to

Create a virtual environment:

```
  python3 -m venv env
```

and activate the virtual environment using:

```
  source env/bin/activate
```

## Install the Ahnlich Client PY

You can install the Ahnlich Python client sdk using either Poetry or pip. Choose one of the methods below:

### Install with Poetry

If you don’t already have Poetry installed, set it up by running:

```
  curl -sSL https://install.python-poetry.org | python3 -
```

Ensure Poetry is available in your PATH:

```
  export PATH="$HOME/.local/bin:$PATH"
```

Install the Ahnlich Python client:

```
  poetry add ahnlich-client-py
```

### Install with pip

If you don’t have pip installed, first install it by running:

```
  sudo apt update  
  sudo apt install python3-pip -y
```

On macOS, you can install pip with `brew install python3` or by downloading Python from python.org

Once pip is available, install the Ahnlich Python client sdk using:

```
  pip3 install ahnlich-client-py
```

## Package Information

The Ahnlich Python client provides a gRPC-based SDK for interacting with Ahnlich-DB (vector storage, similarity search) and Ahnlich-AI (semantic models).

### Modules

* grpclib – async gRPC client library.

* ahnlich_client_py.grpc.db.server – contains server responses for all DB queries.

* ahnlich_client_py.grpc.ai.server – contains server responses for AI queries.

### Initialization

Every request starts by creating a client connection to the Ahnlich server:

```py
import asyncio
from grpclib.client import Channel
from ahnlich_client_py.grpc.services import db_service

async def init_client():

  async with Channel(host="127.0.0.1", port=1369) as channel:

    client = db_service.DbServiceStub(channel)

    # client is now ready for requests

if __name__ == "__main__":

  asyncio.run(init_client())
```