# Ahnlich MCP

An MCP server that exposes [Ahnlich](https://ahnlich.dev/) vector storage and semantic search to MCP-compatible agents.

Ahnlich MCP supports two profiles:

- `db` connects directly to Ahnlich DB and accepts precomputed embeddings.
- `ai` sends raw text through Ahnlich AI, which generates embeddings and stores them in Ahnlich DB.

## Before you start

Ahnlich MCP connects to running Ahnlich services.

| Profile | Required services |
|---|---|
| `ai` | Ahnlich DB on port `1369` and Ahnlich AI on port `1370` |
| `db` | Ahnlich DB on port `1369` |

Follow the [Ahnlich installation guide](https://ahnlich.dev/docs/getting-started/installation/) to start the required services.

The examples below use the `ai` profile. To supply your own embeddings, replace `--profile ai` with `--profile db`.

## Install from PyPI

This is the recommended installation method. Install [uv](https://docs.astral.sh/uv/getting-started/installation/), then use `uvx` to run Ahnlich MCP directly from PyPI.

Verify that the required Ahnlich services are available:

```bash
uvx ahnlich-mcp doctor --profile ai
```

### Claude Desktop

Open **Settings → Developer → Edit Config** and add:

```json
{
  "mcpServers": {
    "ahnlich": {
      "command": "uvx",
      "args": [
        "ahnlich-mcp",
        "--profile",
        "ai"
      ]
    }
  }
}
```

Restart Claude Desktop after saving the configuration.

If Claude Desktop cannot find `uvx`, run `command -v uvx` and use the returned absolute path as `command`.

### Codex

Add the server from your terminal:

```bash
codex mcp add ahnlich -- uvx ahnlich-mcp --profile ai
```

Confirm that it was added:

```bash
codex mcp list
```

You can also use `/mcp` inside Codex to inspect the connection.

## Run with Docker

Use Docker when you want the MCP server and its Python dependencies isolated in a container.

The Ahnlich services must already be running and accessible through their default host ports.

Pull the image:

```bash
docker pull ghcr.io/deven96/ahnlich-mcp:latest
```

### Claude Desktop

Add the following to the Claude Desktop configuration:

```json
{
  "mcpServers": {
    "ahnlich": {
      "command": "docker",
      "args": [
        "run",
        "--rm",
        "-i",
        "--add-host",
        "host.docker.internal:host-gateway",
        "-e",
        "AHNLICH_AI_HOST=host.docker.internal",
        "-e",
        "AHNLICH_AI_PORT=1370",
        "ghcr.io/deven96/ahnlich-mcp:latest",
        "--profile",
        "ai"
      ]
    }
  }
}
```

Restart Claude Desktop after saving the configuration.

### Codex

```bash
codex mcp add ahnlich -- docker run --rm -i \
  --add-host host.docker.internal:host-gateway \
  -e AHNLICH_AI_HOST=host.docker.internal \
  -e AHNLICH_AI_PORT=1370 \
  ghcr.io/deven96/ahnlich-mcp:latest \
  --profile ai
```

## Install from source

Use this method when developing or contributing to Ahnlich MCP.

Clone the repository and install the locked dependencies:

```bash
git clone https://github.com/deven96/ahnlich.git
cd ahnlich/mcp
uv sync --locked --dev
```

Verify the setup:

```bash
uv run ahnlich-mcp doctor --profile ai
```

The stdio command for MCP clients is:

```bash
uv --directory /absolute/path/to/ahnlich/mcp run ahnlich-mcp --profile ai
```

For Claude Desktop:

```json
{
  "mcpServers": {
    "ahnlich": {
      "command": "uv",
      "args": [
        "--directory",
        "/absolute/path/to/ahnlich/mcp",
        "run",
        "ahnlich-mcp",
        "--profile",
        "ai"
      ]
    }
  }
}
```

For Codex:

```bash
codex mcp add ahnlich -- \
  uv --directory /absolute/path/to/ahnlich/mcp \
  run ahnlich-mcp --profile ai
```

## Configuration

Command-line profile selection overrides `AHNLICH_PROFILE`.

| Variable | Default | Description |
|---|---:|---|
| `AHNLICH_PROFILE` | `ai` | Active profile: `ai` or `db` |
| `AHNLICH_DB_HOST` | `127.0.0.1` | Database host |
| `AHNLICH_DB_PORT` | `1369` | Database port |
| `AHNLICH_AI_HOST` | `127.0.0.1` | AI proxy host |
| `AHNLICH_AI_PORT` | `1370` | AI proxy port |
| `AHNLICH_AI_MODEL` | `all-minilm-l6-v2` | Model used by the AI profile |
| `AHNLICH_MCP_READ_ONLY` | `0` | Expose only non-modifying tools |

Supported AI models:

- `all-minilm-l6-v2`
- `all-minilm-l12-v2`
- `bge-base-en-v1.5`
- `bge-large-en-v1.5`
- `jina-embeddings-v2-base-code`

The selected model must also be enabled in `ahnlich-ai` through its
`--supported-models` option. The bundled Compose configuration enables
`all-minilm-l6-v2`.

The bundled Compose configuration uses Ahnlich DB `0.3.2` and Ahnlich AI
`0.4.1` by default. Override `AHNLICH_DB_VERSION` or
`AHNLICH_AI_VERSION` only when testing another compatible release.

Example:

```bash
AHNLICH_PROFILE=db \
AHNLICH_DB_HOST=127.0.0.1 \
AHNLICH_DB_PORT=1369 \
uv run ahnlich-mcp
```

## Read-only mode

Enable strict read-only mode when the MCP client must not modify Ahnlich:

```bash
AHNLICH_MCP_READ_ONLY=1 uv run ahnlich-mcp --profile ai
```

Only these tools are exposed:

- `ping`
- `server_info`
- `list_stores`
- `similarity_search`
- `get_by_metadata`

Mutating tools are omitted from the MCP tool registry.

## Tools

Both profiles expose the same tool names by default. Input schemas differ where embeddings are involved.

| Tool | Description |
|---|---|
| `ping` | Check the configured Ahnlich service |
| `server_info` | Get information about the configured service |
| `create_store` | Create a vector store |
| `list_stores` | List stores |
| `drop_store` | Delete a store and its entries |
| `store_entries` | Store raw text or precomputed embeddings |
| `similarity_search` | Search using raw text or a query embedding |
| `get_by_metadata` | Retrieve entries matching indexed metadata |
| `delete_by_metadata` | Delete entries matching indexed metadata |
| `create_predicate_index` | Index metadata keys |
| `drop_predicate_index` | Remove metadata indexes |

The `db` profile requires `dimension` when creating a store. Its `store_entries` and `similarity_search` tools accept embeddings.

The `ai` profile accepts raw text and uses the configured Ahnlich model to generate embeddings.

Metadata filters use the `metadata_filter` argument. Filtered metadata keys must have predicate indexes.

## Result controls

`list_stores` and `get_by_metadata` accept a `limit` between `1` and `1024`. The default is `50`.

They return a bounded response:

```json
{
  "results": [],
  "truncated": false
}
```

`similarity_search` accepts `top_k` up to `1024`.

Stored embeddings are omitted from DB search and metadata responses by default. Pass `include_embeddings: true` when the vectors are required.

## AI preprocessing

The AI profile supports the following preprocessing modes for `store_entries` and `similarity_search`:

| Value | Behaviour |
|---|---|
| `none` | Send input without model preprocessing |
| `truncate` | Allow Ahnlich to truncate input for the selected model |

The default is `none`.

## Development

Run unit tests:

```bash
uv run pytest tests/unit -v
```

Run integration tests:

```bash
docker compose up -d --wait
uv run pytest tests/integration -v
```

Run the complete test suite:

```bash
uv run pytest -v
```

Run MCP Inspector:

```bash
npx -y @modelcontextprotocol/inspector \
  uv \
  --directory "$(pwd)" \
  run ahnlich-mcp \
  --profile ai
```

## License

MIT