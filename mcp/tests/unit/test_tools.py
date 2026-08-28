from __future__ import annotations

from inspect import signature
from unittest.mock import AsyncMock

import pytest
from mcp.server.fastmcp.exceptions import ToolError
from pydantic import ValidationError

from ahnlich_mcp.backends.ai import AIBackend
from ahnlich_mcp.backends.db import DBBackend
from ahnlich_mcp.tools import (
    EmbeddedEntry,
    MAX_RESULT_LIMIT,
    TOOL_ANNOTATIONS,
    TOOL_ORDER,
    TextEntry,
    build_tools,
)

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

READ_ONLY_TOOL_NAMES = (
    "ping",
    "server_info",
    "list_stores",
    "similarity_search",
    "get_by_metadata",
)


def tool_map(
    backend: AIBackend | DBBackend,
):
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
    assert (
        parameters["dimension"].default
        is parameters["dimension"].empty
    )


def test_ai_search_accepts_text(
    ai_backend: AIBackend,
) -> None:
    parameters = signature(
        tool_map(ai_backend)[
            "similarity_search"
        ]
    ).parameters

    assert "query" in parameters
    assert "query_embedding" not in parameters


def test_db_search_accepts_an_embedding(
    db_backend: DBBackend,
) -> None:
    parameters = signature(
        tool_map(db_backend)[
            "similarity_search"
        ]
    ).parameters

    assert "query_embedding" in parameters
    assert "query" not in parameters
    assert "include_embeddings" in parameters
    assert (
        parameters[
            "include_embeddings"
        ].default
        is False
    )


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

    store_entries = tool_map(
        ai_backend
    )["store_entries"]

    result = await store_entries(
        store_name="documents",
        entries=[
            TextEntry(
                content="Ahnlich stores vectors.",
                metadata={
                    "topic": "vectors",
                },
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
                "content": (
                    "Ahnlich stores vectors."
                ),
                "metadata": {
                    "topic": "vectors",
                },
            }
        ],
        preprocessing="none",
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

    store_entries = tool_map(
        db_backend
    )["store_entries"]

    result = await store_entries(
        store_name="embeddings",
        entries=[
            EmbeddedEntry(
                embedding=[
                    0.1,
                    0.2,
                    0.3,
                ],
                metadata={
                    "source": "test",
                },
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
                "embedding": [
                    0.1,
                    0.2,
                    0.3,
                ],
                "metadata": {
                    "source": "test",
                },
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
    assert (
        set(TOOL_ANNOTATIONS)
        == set(TOOL_ORDER)
    )


def test_tool_annotation_policies() -> None:
    read_only = {
        name
        for name, annotations
        in TOOL_ANNOTATIONS.items()
        if annotations.readOnlyHint
    }
    destructive = {
        name
        for name, annotations
        in TOOL_ANNOTATIONS.items()
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
        for annotations
        in TOOL_ANNOTATIONS.values()
    )


def test_read_only_mode_exposes_only_read_only_tools(
    ai_backend: AIBackend,
    db_backend: DBBackend,
) -> None:
    for backend in (
        ai_backend,
        db_backend,
    ):
        tools = build_tools(
            backend,
            read_only=True,
        )
        names = tuple(
            tool.__name__
            for tool in tools
        )

        assert names == READ_ONLY_TOOL_NAMES
        assert all(
            TOOL_ANNOTATIONS[
                name
            ].readOnlyHint
            is True
            for name in names
        )


@pytest.mark.asyncio
async def test_list_stores_returns_bounded_results(
    db_backend: DBBackend,
) -> None:
    stores = [
        {"name": "one"},
        {"name": "two"},
        {"name": "three"},
    ]
    db_backend.list_stores = AsyncMock(
        return_value=stores
    )

    list_stores = tool_map(
        db_backend
    )["list_stores"]

    result = await list_stores(
        limit=2
    )

    assert result == {
        "results": stores[:2],
        "truncated": True,
    }


@pytest.mark.asyncio
async def test_get_by_metadata_returns_bounded_results(
    db_backend: DBBackend,
) -> None:
    entries = [
        {
            "dimension": 3,
            "metadata": {"id": "1"},
        },
        {
            "dimension": 3,
            "metadata": {"id": "2"},
        },
    ]
    db_backend.get_by_metadata = AsyncMock(
        return_value=entries
    )

    get_by_metadata = tool_map(
        db_backend
    )["get_by_metadata"]

    result = await get_by_metadata(
        store_name="documents",
        metadata_filter={
            "status": "active",
        },
        limit=1,
        include_embeddings=True,
    )

    assert result == {
        "results": entries[:1],
        "truncated": True,
    }

    db_backend.get_by_metadata.assert_awaited_once_with(
        store_name="documents",
        metadata_filter={
            "status": "active",
        },
        include_embeddings=True,
    )


@pytest.mark.parametrize(
    "limit",
    [
        0,
        MAX_RESULT_LIMIT + 1,
        True,
    ],
)
@pytest.mark.asyncio
async def test_list_stores_rejects_invalid_limit(
    db_backend: DBBackend,
    limit: object,
) -> None:
    db_backend.list_stores = AsyncMock(
        return_value=[]
    )
    list_stores = tool_map(
        db_backend
    )["list_stores"]

    with pytest.raises(
        ToolError,
        match="limit",
    ):
        await list_stores(
            limit=limit
        )

    db_backend.list_stores.assert_not_awaited()


@pytest.mark.asyncio
async def test_db_search_forwards_embedding_option(
    db_backend: DBBackend,
) -> None:
    db_backend.similarity_search = AsyncMock(
        return_value=[]
    )
    similarity_search = tool_map(
        db_backend
    )["similarity_search"]

    await similarity_search(
        store_name="documents",
        query_embedding=[
            0.1,
            0.2,
            0.3,
        ],
        include_embeddings=True,
    )

    db_backend.similarity_search.assert_awaited_once_with(
        store_name="documents",
        query_embedding=[
            0.1,
            0.2,
            0.3,
        ],
        top_k=5,
        algorithm="cosine",
        metadata_filter=None,
        include_embeddings=True,
    )


def test_metadata_filter_has_consistent_wire_name(
    ai_backend: AIBackend,
    db_backend: DBBackend,
) -> None:
    for backend in (
        ai_backend,
        db_backend,
    ):
        tools = tool_map(backend)

        for tool_name in (
            "similarity_search",
            "get_by_metadata",
            "delete_by_metadata",
        ):
            parameters = signature(
                tools[tool_name]
            ).parameters

            assert (
                "metadata_filter"
                in parameters
            )
            assert "filter" not in parameters


def test_preprocessing_is_exposed_only_by_ai_tools(
    ai_backend: AIBackend,
    db_backend: DBBackend,
) -> None:
    ai_tools = tool_map(ai_backend)
    db_tools = tool_map(db_backend)

    for tool_name in (
        "store_entries",
        "similarity_search",
    ):
        ai_parameters = signature(
            ai_tools[tool_name]
        ).parameters
        db_parameters = signature(
            db_tools[tool_name]
        ).parameters

        assert (
            "preprocessing"
            in ai_parameters
        )
        assert (
            ai_parameters[
                "preprocessing"
            ].default
            == "none"
        )
        assert (
            "preprocessing"
            not in db_parameters
        )


@pytest.mark.asyncio
async def test_ai_search_forwards_preprocessing(
    ai_backend: AIBackend,
) -> None:
    ai_backend.similarity_search = AsyncMock(
        return_value=[]
    )
    similarity_search = tool_map(
        ai_backend
    )["similarity_search"]

    await similarity_search(
        store_name="documents",
        query="A long document",
        preprocessing="truncate",
    )

    ai_backend.similarity_search.assert_awaited_once_with(
        store_name="documents",
        query="A long document",
        top_k=5,
        algorithm="cosine",
        metadata_filter=None,
        preprocessing="truncate",
    )