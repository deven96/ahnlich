from __future__ import annotations

from mcp.server.fastmcp import FastMCP
from ahnlich_mcp.tools import MCP_TOOLS

mcp = FastMCP(
    name="ahnlich-mcp",
    instructions=(
        "Store and search text using the Ahnlich vector database. "
        "Use create_store before ingestion. Use store_content or "
        "upsert_content to index text, similarity_search to search by "
        "meaning, and metadata tools for structured filtering. Metadata "
        "fields used in filters must have predicate indexes."
    ),
    json_response=True,
)

for tool in MCP_TOOLS:
    mcp.tool()(tool)

def main() -> None:
    """Run the Ahnlich MCP server using stdio transport."""
    mcp.run()

if __name__ == "__main__":
    main()