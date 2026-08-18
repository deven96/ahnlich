from __future__ import annotations

import json
import os
import sys
from collections.abc import AsyncIterator
from contextlib import asynccontextmanager
from pathlib import Path
from typing import Any
from uuid import uuid4

import pytest
from mcp import ClientSession, StdioServerParameters
from mcp.client.stdio import stdio_client
from mcp.types import CallToolResult, TextContent

pytestmark = pytest.mark.integration

PROJECT_ROOT = Path(__file__).parents[2]


@asynccontextmanager
async def mcp_session(
    profile: str,
) -> AsyncIterator[ClientSession]:
    environment = os.environ.copy()
    environment.setdefault(
        "AHNLICH_DB_HOST",
        "127.0.0.1",
    )
    environment.setdefault(
        "AHNLICH_DB_PORT",
        "1369",
    )
    environment.setdefault(
        "AHNLICH_AI_HOST",
        "127.0.0.1",
    )
    environment.setdefault(
        "AHNLICH_AI_PORT",
        "1370",
    )

    parameters = StdioServerParameters(
        command=sys.executable,
        args=[
            "-m",
            "ahnlich_mcp.server",
            "--profile",
            profile,
        ],
        cwd=PROJECT_ROOT,
        env=environment,
    )

    async with stdio_client(parameters) as (
        read_stream,
        write_stream,
    ):
        async with ClientSession(
            read_stream,
            write_stream,
        ) as session:
            await session.initialize()
            yield session


def decode_result(
    result: CallToolResult,
) -> Any:
    assert result.isError is not True, result

    if result.structuredContent is not None:
        structured = result.structuredContent

        if set(structured) == {"result"}:
            return structured["result"]

        return structured

    decoded_blocks = [
        json.loads(block.text)
        for block in result.content
        if isinstance(block, TextContent)
    ]

    if not decoded_blocks:
        raise AssertionError(
            "Tool result did not contain JSON content"
        )

    if len(decoded_blocks) == 1:
        return decoded_blocks[0]

    return decoded_blocks


async def call_tool(
    session: ClientSession,
    name: str,
    arguments: dict[str, Any],
) -> Any:
    result = await session.call_tool(
        name,
        arguments,
    )
    payload = decode_result(result)

    if isinstance(payload, dict):
        assert payload.get("status") != "error", payload
        assert "error" not in payload, payload

    return payload


def unique_store_name(profile: str) -> str:
    return f"mcp_{profile}_{uuid4().hex}"


@pytest.mark.asyncio
async def test_ai_profile_over_stdio() -> None:
    store_name = unique_store_name("ai")

    async with mcp_session("ai") as session:
        tools = await session.list_tools()
        tool_names = {
            tool.name for tool in tools.tools
        }

        assert "store_entries" in tool_names
        assert "similarity_search" in tool_names

        create_store = next(
            tool
            for tool in tools.tools
            if tool.name == "create_store"
        )
        properties = create_store.inputSchema[
            "properties"
        ]

        assert "dimension" not in properties

        ping = await call_tool(
            session,
            "ping",
            {},
        )
        assert ping["profile"] == "ai"
        assert ping["available"] is True

        try:
            created = await call_tool(
                session,
                "create_store",
                {
                    "store_name": store_name,
                    "predicate_keys": ["topic"],
                },
            )
            assert created["status"] == "created"

            stored = await call_tool(
                session,
                "store_entries",
                {
                    "store_name": store_name,
                    "entries": [
                        {
                            "content": (
                                "Saturn is a planet with "
                                "a prominent ring system."
                            ),
                            "metadata": {
                                "topic": "astronomy",
                            },
                        },
                        {
                            "content": (
                                "Fresh basil is commonly used "
                                "in Italian cooking."
                            ),
                            "metadata": {
                                "topic": "cooking",
                            },
                        },
                    ],
                },
            )
            assert stored["inserted"] == 2

            results = await call_tool(
                session,
                "similarity_search",
                {
                    "store_name": store_name,
                    "query": "Which planet has rings?",
                    "top_k": 1,
                },
            )

            assert len(results) == 1
            assert (
                results[0]["metadata"]["topic"]
                == "astronomy"
            )

            matching = await call_tool(
                session,
                "get_by_metadata",
                {
                    "store_name": store_name,
                    "filter": {
                        "topic": "astronomy",
                    },
                },
            )

            assert len(matching) == 1

        finally:
            await session.call_tool(
                "drop_store",
                {
                    "store_name": store_name,
                    "error_if_not_exists": False,
                },
            )


@pytest.mark.asyncio
async def test_db_profile_over_stdio() -> None:
    store_name = unique_store_name("db")

    async with mcp_session("db") as session:
        tools = await session.list_tools()

        create_store = next(
            tool
            for tool in tools.tools
            if tool.name == "create_store"
        )
        properties = create_store.inputSchema[
            "properties"
        ]

        assert "dimension" in properties

        search = next(
            tool
            for tool in tools.tools
            if tool.name == "similarity_search"
        )
        search_properties = search.inputSchema[
            "properties"
        ]

        assert "query_embedding" in search_properties
        assert "query" not in search_properties

        ping = await call_tool(
            session,
            "ping",
            {},
        )
        assert ping["profile"] == "db"
        assert ping["available"] is True

        try:
            created = await call_tool(
                session,
                "create_store",
                {
                    "store_name": store_name,
                    "dimension": 3,
                    "predicate_keys": ["topic"],
                },
            )

            assert created["status"] == "created"
            assert created["dimension"] == 3

            stored = await call_tool(
                session,
                "store_entries",
                {
                    "store_name": store_name,
                    "entries": [
                        {
                            "embedding": [1.0, 0.0, 0.0],
                            "metadata": {
                                "topic": "red",
                            },
                        },
                        {
                            "embedding": [0.0, 1.0, 0.0],
                            "metadata": {
                                "topic": "green",
                            },
                        },
                    ],
                },
            )

            assert stored["inserted"] == 2

            results = await call_tool(
                session,
                "similarity_search",
                {
                    "store_name": store_name,
                    "query_embedding": [
                        0.9,
                        0.1,
                        0.0,
                    ],
                    "top_k": 1,
                },
            )

            assert len(results) == 1
            assert results[0]["metadata"]["topic"] == "red"

            matching = await call_tool(
                session,
                "get_by_metadata",
                {
                    "store_name": store_name,
                    "filter": {
                        "topic": "green",
                    },
                },
            )

            assert len(matching) == 1

        finally:
            await session.call_tool(
                "drop_store",
                {
                    "store_name": store_name,
                    "error_if_not_exists": False,
                },
            )

@pytest.mark.asyncio
async def test_tool_error_sets_protocol_error_flag() -> None:
    missing_store = unique_store_name("missing")

    async with mcp_session("db") as session:
        result = await session.call_tool(
            "get_by_metadata",
            {
                "store_name": missing_store,
                "filter": {
                    "topic": "missing",
                },
            },
        )

    error_text = " ".join(
        block.text
        for block in result.content
        if isinstance(block, TextContent)
    )

    assert result.isError is True
    assert '"status": "error"' in error_text
    assert missing_store in error_text
    assert "create_store" in error_text

@pytest.mark.asyncio
async def test_tool_annotations_over_stdio() -> None:
    async with mcp_session("db") as session:
        response = await session.list_tools()

    tools = {
        tool.name: tool
        for tool in response.tools
    }

    assert all(
        tool.annotations is not None
        for tool in tools.values()
    )

    read_only = {
        name
        for name, tool in tools.items()
        if (
            tool.annotations is not None
            and tool.annotations.readOnlyHint
        )
    }
    destructive = {
        name
        for name, tool in tools.items()
        if (
            tool.annotations is not None
            and tool.annotations.destructiveHint
        )
    }

    assert read_only == {
        "ping",
        "server_info",
        "list_stores",
        "similarity_search",
        "get_by_metadata",
    }
    assert destructive == {
        "drop_store",
        "delete_by_metadata",
        "drop_predicate_index",
    }
    assert all(
        tool.annotations is not None
        and tool.annotations.openWorldHint is False
        for tool in tools.values()
    )