from __future__ import annotations

import asyncio
from uuid import uuid4

import pytest
import pytest_asyncio

from ahnlich_mcp.server import mcp
from ahnlich_mcp.tools import (
    MCP_TOOLS,
    client,
    create_predicate_index,
    create_store,
    delete_by_metadata,
    drop_predicate_index,
    drop_store,
    get_by_metadata,
    list_stores,
    ping,
    similarity_search,
    store_content,
    upsert_content,
)


EXPECTED_TOOLS = {
    "ping",
    "server_info",
    "create_store",
    "list_stores",
    "drop_store",
    "store_content",
    "upsert_content",
    "similarity_search",
    "get_by_metadata",
    "delete_by_metadata",
    "create_predicate_index",
    "drop_predicate_index",
}


def unique_store_name(prefix: str) -> str:
    return f"{prefix}_{uuid4().hex}"


@pytest_asyncio.fixture(autouse=True)
async def close_channels_after_test():
    yield
    await client.close()


@pytest_asyncio.fixture
async def store_name():
    name = unique_store_name("test")

    yield name

    await drop_store(
        store_name=name,
        error_if_not_exists=False,
    )


def test_tool_registry_contains_every_exported_tool() -> None:
    assert {tool.__name__ for tool in MCP_TOOLS} == EXPECTED_TOOLS


def test_server_registers_every_tool() -> None:
    registered = asyncio.run(mcp.list_tools())
    assert {tool.name for tool in registered} == EXPECTED_TOOLS


@pytest.mark.asyncio
async def test_ping() -> None:
    result = await ping()

    assert result["status"] == "ok"
    assert result["db"] is True
    assert result["ai"] is True


@pytest.mark.asyncio
async def test_create_and_list_stores(
    store_name: str,
) -> None:
    created = await create_store(store_name)

    assert created["status"] == "created"
    assert created["store_name"] == store_name

    stores = await list_stores()

    assert isinstance(stores, list)
    assert store_name in {
        store["name"] for store in stores
    }

    dropped = await drop_store(store_name)

    assert dropped["status"] == "dropped"


@pytest.mark.asyncio
async def test_store_and_search(
    store_name: str,
) -> None:
    created = await create_store(store_name)
    assert created["status"] == "created"

    stored = await store_content(
        store_name=store_name,
        entries=[
            {
                "content": (
                    "Neural networks learn representations from data."
                ),
                "metadata": {"topic": "machine-learning"},
            },
            {
                "content": (
                    "Tomatoes and basil are used in pasta sauce."
                ),
                "metadata": {"topic": "cooking"},
            },
            {
                "content": (
                    "Jupiter is the largest planet in the solar system."
                ),
                "metadata": {"topic": "astronomy"},
            },
        ],
    )

    assert stored["inserted"] == 3

    results = await similarity_search(
        store_name=store_name,
        query="How do artificial neural networks learn?",
        top_k=3,
    )

    assert isinstance(results, list)
    assert len(results) == 3
    assert (
        results[0]["metadata"]["topic"]
        == "machine-learning"
    )


@pytest.mark.asyncio
async def test_upsert(
    store_name: str,
) -> None:
    created = await create_store(
        store_name=store_name,
        predicate_keys=["version"],
    )
    assert created["status"] == "created"

    original = await store_content(
        store_name=store_name,
        entries=[
            {
                "content": "A document that will be updated.",
                "metadata": {"version": "one"},
            }
        ],
    )
    assert original["inserted"] == 1

    updated = await upsert_content(
        store_name=store_name,
        entries=[
            {
                "content": "A document that will be updated.",
                "metadata": {"version": "two"},
            }
        ],
    )

    assert updated["updated"] == 1
    assert updated["inserted"] == 0

    version_two = await get_by_metadata(
        store_name=store_name,
        filter={"version": "two"},
    )

    assert isinstance(version_two, list)
    assert len(version_two) == 1
    assert version_two[0]["metadata"]["version"] == "two"


@pytest.mark.asyncio
async def test_metadata_filter(
    store_name: str,
) -> None:
    created = await create_store(
        store_name=store_name,
        predicate_keys=["category"],
    )
    assert created["status"] == "created"

    stored = await store_content(
        store_name=store_name,
        entries=[
            {
                "content": "A guide to Python programming.",
                "metadata": {"category": "code"},
            },
            {
                "content": "A guide to Italian cooking.",
                "metadata": {"category": "food"},
            },
            {
                "content": "Python type checking with mypy.",
                "metadata": {"category": "code"},
            },
        ],
    )
    assert stored["inserted"] == 3

    results = await similarity_search(
        store_name=store_name,
        query="help with programming",
        top_k=5,
        filter={"category": "code"},
    )

    assert isinstance(results, list)
    assert len(results) == 2
    assert all(
        result["metadata"]["category"] == "code"
        for result in results
    )


@pytest.mark.asyncio
async def test_predicate_index(
    store_name: str,
) -> None:
    created_store = await create_store(store_name)
    assert created_store["status"] == "created"

    created_index = await create_predicate_index(
        store_name=store_name,
        keys=["author"],
    )

    assert created_index["status"] == "created"
    assert created_index["created"] == 1

    dropped_index = await drop_predicate_index(
        store_name=store_name,
        keys=["author"],
    )

    assert dropped_index["status"] == "dropped"
    assert dropped_index["deleted"] == 1


@pytest.mark.asyncio
async def test_delete_by_metadata(
    store_name: str,
) -> None:
    created = await create_store(
        store_name=store_name,
        predicate_keys=["status"],
    )
    assert created["status"] == "created"

    stored = await store_content(
        store_name=store_name,
        entries=[
            {
                "content": "Keep this document.",
                "metadata": {"status": "active"},
            },
            {
                "content": "Delete this document.",
                "metadata": {"status": "archived"},
            },
            {
                "content": "Delete this old note too.",
                "metadata": {"status": "archived"},
            },
        ],
    )
    assert stored["inserted"] == 3

    deleted = await delete_by_metadata(
        store_name=store_name,
        filter={"status": "archived"},
    )

    assert deleted["deleted"] == 2

    archived = await get_by_metadata(
        store_name=store_name,
        filter={"status": "archived"},
    )

    assert archived == []

    active = await get_by_metadata(
        store_name=store_name,
        filter={"status": "active"},
    )

    assert isinstance(active, list)
    assert len(active) == 1