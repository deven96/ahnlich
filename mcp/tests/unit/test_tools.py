from __future__ import annotations

from inspect import signature
from unittest.mock import AsyncMock

import pytest
from pydantic import ValidationError

from ahnlich_mcp.backends.ai import AIBackend
from ahnlich_mcp.backends.db import DBBackend
from ahnlich_mcp.tools import (
    EmbeddedEntry,
    TextEntry,
    build_tools,
    TOOL_ANNOTATIONS,
    TOOL_ORDER,
)

from mcp.server.fastmcp.exceptions import ToolError

EXPECTED_TOOLS = (
    "ping",
    "server_info",
    "create_store",
    "list_stores",
    "drop_store",
    "store_entries",
    "similarity_search",
    "get_by_metadata",
    "delete_by_metadata",
    "create_predicate_index",
    "drop_predicate_index",
)


def tool_map(backend: AIBackend | DBBackend):
    return {
        tool.__name__: tool
        for tool in build_tools(backend)
    }


@pytest.fixture
def ai_backend() -> AIBackend:
    return AIBackend(
        host="127.0.0.1",
        port=1370,
        model="all-minilm-l6-v2",
    )


@pytest.fixture
def db_backend() -> DBBackend:
    return DBBackend(
        host="127.0.0.1",
        port=1369,
    )


def test_profiles_expose_the_same_tool_names(
    ai_backend: AIBackend,
    db_backend: DBBackend,
) -> None:
    ai_names = tuple(
        tool.__name__
        for tool in build_tools(ai_backend)
    )
    db_names = tuple(
        tool.__name__
        for tool in build_tools(db_backend)
    )

    assert ai_names == EXPECTED_TOOLS
    assert db_names == EXPECTED_TOOLS


def test_ai_create_store_does_not_require_dimension(
    ai_backend: AIBackend,
) -> None:
    parameters = signature(
        tool_map(ai_backend)["create_store"]
    ).parameters

    assert "dimension" not in parameters
    assert "predicate_keys" in parameters


def test_db_create_store_requires_dimension(
    db_backend: DBBackend,
) -> None:
    parameters = signature(
        tool_map(db_backend)["create_store"]
    ).parameters

    assert "dimension" in parameters
    assert parameters["dimension"].default is parameters[
        "dimension"
    ].empty


def test_ai_search_accepts_text(
    ai_backend: AIBackend,
) -> None:
    parameters = signature(
        tool_map(ai_backend)["similarity_search"]
    ).parameters

    assert "query" in parameters
    assert "query_embedding" not in parameters


def test_db_search_accepts_an_embedding(
    db_backend: DBBackend,
) -> None:
    parameters = signature(
        tool_map(db_backend)["similarity_search"]
    ).parameters

    assert "query_embedding" in parameters
    assert "query" not in parameters


def test_text_entry_rejects_empty_content() -> None:
    with pytest.raises(ValidationError):
        TextEntry(content="")


def test_embedded_entry_rejects_empty_embedding() -> None:
    with pytest.raises(ValidationError):
        EmbeddedEntry(embedding=[])


@pytest.mark.asyncio
async def test_ai_store_entries_converts_models_to_dicts(
    ai_backend: AIBackend,
) -> None:
    ai_backend.store_entries = AsyncMock(
        return_value={
            "inserted": 1,
            "updated": 0,
        }
    )

    store_entries = tool_map(ai_backend)["store_entries"]

    result = await store_entries(
        store_name="documents",
        entries=[
            TextEntry(
                content="Ahnlich stores vectors.",
                metadata={"topic": "vectors"},
            )
        ],
    )

    assert result == {
        "inserted": 1,
        "updated": 0,
    }

    ai_backend.store_entries.assert_awaited_once_with(
        store_name="documents",
        entries=[
            {
                "content": "Ahnlich stores vectors.",
                "metadata": {"topic": "vectors"},
            }
        ],
    )


@pytest.mark.asyncio
async def test_db_store_entries_converts_models_to_dicts(
    db_backend: DBBackend,
) -> None:
    db_backend.store_entries = AsyncMock(
        return_value={
            "inserted": 1,
            "updated": 0,
        }
    )

    store_entries = tool_map(db_backend)["store_entries"]

    result = await store_entries(
        store_name="embeddings",
        entries=[
            EmbeddedEntry(
                embedding=[0.1, 0.2, 0.3],
                metadata={"source": "test"},
            )
        ],
    )

    assert result == {
        "inserted": 1,
        "updated": 0,
    }

    db_backend.store_entries.assert_awaited_once_with(
        store_name="embeddings",
        entries=[
            {
                "embedding": [0.1, 0.2, 0.3],
                "metadata": {"source": "test"},
            }
        ],
    )

@pytest.mark.asyncio
async def test_ping_returns_unavailable_diagnostic(
    db_backend: DBBackend,
) -> None:
    db_backend.ping = AsyncMock(
        return_value=False
    )
    ping = tool_map(db_backend)["ping"]

    result = await ping()

    assert result["status"] == "error"
    assert result["available"] is False
    assert "suggested_action" in result


@pytest.mark.asyncio
async def test_ping_converts_unexpected_error_to_tool_error(
    db_backend: DBBackend,
) -> None:
    db_backend.ping = AsyncMock(
        side_effect=RuntimeError(
            "unexpected failure"
        )
    )
    ping = tool_map(db_backend)["ping"]

    with pytest.raises(
        ToolError,
        match="unexpected failure",
    ):
        await ping()

def test_every_tool_declares_annotations() -> None:
    assert set(TOOL_ANNOTATIONS) == set(TOOL_ORDER)


def test_tool_annotation_policies() -> None:
    read_only = {
        name
        for name, annotations in TOOL_ANNOTATIONS.items()
        if annotations.readOnlyHint
    }
    destructive = {
        name
        for name, annotations in TOOL_ANNOTATIONS.items()
        if annotations.destructiveHint
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
        annotations.openWorldHint is False
        for annotations in TOOL_ANNOTATIONS.values()
    )