# Ahnlich MCP

An MCP server that exposes the
[Ahnlich](https://ahnlich.dev/) vector database to MCP-compatible agents.

It provides composable tools for creating vector stores, indexing text,
semantic search, metadata filtering, and data management. Ahnlich runs
locally and does not require cloud credentials or API keys.

## Requirements

- [uv](https://docs.astral.sh/uv/)
- Docker with Docker Compose
- Python 3.11, managed automatically by uv

## Quickstart

Start Ahnlich DB and the Ahnlich AI proxy:

```bash
docker compose up -d
```

Create the project environment and install dependencies:

```bash
uv sync
```

Run the MCP server:

```bash
uv run ahnlich-mcp
```

The MCP server uses stdio transport. It does not print application output to
stdout because stdout is reserved for MCP JSON-RPC messages.

## Configuration

The following environment variables configure the Ahnlich connections:

| Variable | Default | Description |
|---|---:|---|
| `AHNLICH_DB_HOST` | `127.0.0.1` | Ahnlich DB host |
| `AHNLICH_DB_PORT` | `1369` | Ahnlich DB port |
| `AHNLICH_AI_HOST` | `127.0.0.1` | Ahnlich AI proxy host |
| `AHNLICH_AI_PORT` | `1370` | Ahnlich AI proxy port |

For example:

```bash
AHNLICH_DB_HOST=127.0.0.1 \
AHNLICH_DB_PORT=1369 \
AHNLICH_AI_HOST=127.0.0.1 \
AHNLICH_AI_PORT=1370 \
uv run ahnlich-mcp
```

## MCP client configuration

Run the server through uv from the project directory:

```json
{
  "mcpServers": {
    "ahnlich": {
      "command": "uv",
      "args": [
        "--directory",
        "/absolute/path/to/ahnlich-mcp",
        "run",
        "ahnlich-mcp"
      ],
      "description": "Semantic search and vector storage with Ahnlich"
    }
  }
}
```

If the MCP client cannot find `uv`, locate its absolute path:

```bash
which uv
```

Then use that path as the command:

```json
{
  "mcpServers": {
    "ahnlich": {
      "command": "/home/username/.local/bin/uv",
      "args": [
        "--directory",
        "/absolute/path/to/ahnlich-mcp",
        "run",
        "ahnlich-mcp"
      ],
      "description": "Semantic search and vector storage with Ahnlich"
    }
  }
}
```

Environment variables can also be supplied by the MCP client:

```json
{
  "mcpServers": {
    "ahnlich": {
      "command": "uv",
      "args": [
        "--directory",
        "/absolute/path/to/ahnlich-mcp",
        "run",
        "ahnlich-mcp"
      ],
      "env": {
        "AHNLICH_DB_HOST": "127.0.0.1",
        "AHNLICH_DB_PORT": "1369",
        "AHNLICH_AI_HOST": "127.0.0.1",
        "AHNLICH_AI_PORT": "1370"
      }
    }
  }
}
```

## Tools

| Tool | Description |
|---|---|
| `ping` | Check the DB and AI proxy connections |
| `server_info` | Get AI proxy version and configuration |
| `create_store` | Create a text vector store |
| `list_stores` | List existing stores |
| `drop_store` | Delete a store and its entries |
| `store_content` | Embed and store text with metadata |
| `upsert_content` | Insert new content or update duplicate content |
| `similarity_search` | Search by natural-language meaning |
| `get_by_metadata` | Retrieve entries matching metadata |
| `delete_by_metadata` | Delete entries matching metadata |
| `create_predicate_index` | Index metadata keys for filtering |
| `drop_predicate_index` | Remove metadata indexes |

## Example workflow

Ask your MCP-compatible agent:

> Scan my Downloads folder and index its files so I can search them
> semantically.

The agent can perform the following workflow.

### 1. Read the directory

The agent reads the directory using its own filesystem capabilities. The MCP
server does not read files or scan directories itself.

### 2. Create a vector store

The agent calls `create_store`:

```json
{
  "store_name": "my_files",
  "predicate_keys": [
    "extension",
    "directory"
  ]
}
```

The predicate indexes allow later searches to filter by the `extension` and
`directory` metadata fields.

### 3. Store file descriptions

The agent calls `store_content` in batches:

```json
{
  "store_name": "my_files",
  "entries": [
    {
      "content": "Machine learning paper about transformer models",
      "metadata": {
        "filename": "transformers.pdf",
        "extension": ".pdf",
        "directory": "Downloads"
      }
    },
    {
      "content": "Notes about planning a summer holiday",
      "metadata": {
        "filename": "holiday-notes.txt",
        "extension": ".txt",
        "directory": "Downloads"
      }
    }
  ]
}
```

Ahnlich's AI proxy automatically embeds the text before storing it.

### 4. Search semantically

The agent calls `similarity_search`:

```json
{
  "store_name": "my_files",
  "query": "machine learning papers",
  "top_k": 5
}
```

This searches by meaning rather than requiring exact keyword matches.

### 5. Filter by metadata

The agent calls `get_by_metadata`:

```json
{
  "store_name": "my_files",
  "filter": {
    "extension": ".pdf"
  }
}
```

The agent can also combine semantic search with metadata filtering:

```json
{
  "store_name": "my_files",
  "query": "technical documents",
  "top_k": 5,
  "filter": {
    "extension": ".pdf"
  }
}
```

The MCP server provides vector storage and search primitives. The agent
supplies filesystem access, workflow composition, and reasoning.

## Tool examples

### Check server health

```json
{}
```

Tool: `ping`

Expected response:

```json
{
  "status": "ok",
  "db": true,
  "ai": true
}
```

### Create a store

Tool: `create_store`

```json
{
  "store_name": "research_notes",
  "model": "all-minilm-l6-v2",
  "predicate_keys": [
    "topic",
    "source"
  ]
}
```

### Store content

Tool: `store_content`

```json
{
  "store_name": "research_notes",
  "entries": [
    {
      "content": "Transformers use self-attention to process sequences.",
      "metadata": {
        "topic": "machine-learning",
        "source": "notes"
      }
    }
  ]
}
```

Metadata keys and values must be strings.

### Search content

Tool: `similarity_search`

```json
{
  "store_name": "research_notes",
  "query": "How do attention-based neural networks work?",
  "top_k": 5,
  "algorithm": "cosine"
}
```

Supported similarity algorithms are:

- `cosine`
- `euclidean`
- `dot_product`

### Update existing content

Tool: `upsert_content`

```json
{
  "store_name": "research_notes",
  "entries": [
    {
      "content": "Transformers use self-attention to process sequences.",
      "metadata": {
        "topic": "deep-learning",
        "source": "revised-notes"
      }
    }
  ]
}
```

### Delete content by metadata

Tool: `delete_by_metadata`

```json
{
  "store_name": "research_notes",
  "filter": {
    "source": "revised-notes"
  }
}
```

### Drop a store

Tool: `drop_store`

```json
{
  "store_name": "research_notes"
}
```

Dropping a store permanently removes the store and all its entries.

## Scope

This server intentionally exposes Ahnlich primitives instead of implementing
workflow-specific tools.

It does not:

- Read or modify files.
- Scan directories.
- Move or organize files.
- Compose application-specific workflows.
- Provide image or audio embedding.
- Provide authentication or remote HTTP transport.
- Cache Ahnlich results.

MCP agents compose these tools with their own filesystem access and reasoning.

## Demo

<!-- Add demo GIF here. -->

![Ahnlich MCP demo](docs/demo.gif)

## Development

Install the project and development dependencies:

```bash
uv sync
```

Start Ahnlich:

```bash
docker compose up -d
```

Check the container status:

```bash
docker compose ps
```

Run the integration tests:

```bash
uv run pytest tests/ -v
```

Run the MCP server through its installed command:

```bash
uv run ahnlich-mcp
```

Run it directly as a Python module:

```bash
uv run python -m ahnlich_mcp.server
```

Stop Ahnlich:

```bash
docker compose down
```

## Testing

The integration tests require Ahnlich DB and Ahnlich AI to be running on the
configured ports.

Run all tests:

```bash
uv run pytest tests/ -v
```

Run one test:

```bash
uv run pytest tests/test_tools.py::test_ping -v
```

Run tests matching a name:

```bash
uv run pytest tests/ -v -k similarity
```

## Troubleshooting

### The server cannot connect to Ahnlich

Check the containers:

```bash
docker compose ps
```

Inspect their logs:

```bash
docker compose logs ahnlich_db
docker compose logs ahnlich_ai
```

Restart the services:

```bash
docker compose restart
```

### The MCP client cannot find uv

Find the absolute path:

```bash
which uv
```

Use the returned path as the `command` in the MCP client configuration.

### Dependencies are out of sync

Run:

```bash
uv lock
uv sync
```

### Recreate the environment

Remove the existing environment and let uv recreate it:

```bash
rm -rf .venv
uv sync
```

## Documentation

- [Ahnlich documentation](https://ahnlich.dev/docs/overview)
- [Ahnlich GitHub repository](https://github.com/deven96/ahnlich)
- [Ahnlich Python SDK](https://ahnlich.dev/docs/client-libraries/python/)
- [Model Context Protocol](https://modelcontextprotocol.io/introduction)
- [uv documentation](https://docs.astral.sh/uv/)

## License

MIT