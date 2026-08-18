from __future__ import annotations

from typing import Any
from uuid import uuid4

import pytest
import pytest_asyncio

from ahnlich_mcp.backends.ai import AIBackend

pytestmark = pytest.mark.integration


def unique_store_name(prefix: str) -> str:
    return f"{prefix}_{uuid4().hex}"


@pytest_asyncio.fixture
async def backend() -> AIBackend:
    ai_backend = AIBackend(
        host="127.0.0.1",
        port=1370,
        model="all-minilm-l6-v2",
    )

    yield ai_backend

    await ai_backend.close()


@pytest_asyncio.fixture
async def store_name(
    backend: AIBackend,
) -> str:
    name = unique_store_name("ai_backend_test")

    yield name

    await backend.drop_store(
        store_name=name,
        error_if_not_exists=False,
    )


def test_backend_configuration() -> None:
    backend = AIBackend(
        host="ai.internal",
        port=9000,
        model="all-minilm-l6-v2",
    )

    assert backend.profile_name == "ai"
    assert backend.service_name == "ahnlich-ai"
    assert backend.endpoint == "ai.internal:9000"
    assert backend.model_name == "all-minilm-l6-v2"


def test_unsupported_model_is_rejected() -> None:
    with pytest.raises(
        ValueError,
        match="Unsupported AI model",
    ):
        AIBackend(
            host="127.0.0.1",
            port=1370,
            model="unknown-model",
        )


def test_build_entries() -> None:
    backend = AIBackend(
        host="127.0.0.1",
        port=1370,
        model="all-minilm-l6-v2",
    )

    entries = backend._build_entries(
        [
            {
                "content": "A document about astronomy.",
                "metadata": {
                    "topic": "astronomy",
                },
            }
        ]
    )

    assert len(entries) == 1
    assert entries[0].key.raw_string == (
        "A document about astronomy."
    )
    assert (
        entries[0]
        .value
        .value["topic"]
        .raw_string
        == "astronomy"
    )


@pytest.mark.parametrize(
    "entries",
    [
        [],
        ["not-a-dictionary"],
        [{}],
        [{"content": ""}],
        [{"content": 123}],
        [
            {
                "content": "Valid content",
                "metadata": "not-a-dictionary",
            }
        ],
    ],
)
def test_invalid_entries(
    entries: list[Any],
) -> None:
    backend = AIBackend(
        host="127.0.0.1",
        port=1370,
        model="all-minilm-l6-v2",
    )

    with pytest.raises(ValueError):
        backend._build_entries(entries)


@pytest.mark.asyncio
async def test_ping(
    backend: AIBackend,
) -> None:
    assert await backend.ping() is True


@pytest.mark.asyncio
async def test_server_info(
    backend: AIBackend,
) -> None:
    result = await backend.server_info()

    assert result["profile"] == "ai"
    assert result["service"] == "ahnlich-ai"
    assert result["model"] == "all-minilm-l6-v2"
    assert result["version"]


@pytest.mark.asyncio
async def test_create_and_list_store(
    backend: AIBackend,
    store_name: str,
) -> None:
    await backend.create_store(
        store_name=store_name,
        predicate_keys=["category"],
        error_if_exists=True,
    )

    stores = await backend.list_stores()

    matching_store = next(
        store
        for store in stores
        if store["name"] == store_name
    )

    assert matching_store["name"] == store_name
    assert (
        matching_store["query_model"]
        == "all-minilm-l6-v2"
    )
    assert (
        matching_store["index_model"]
        == "all-minilm-l6-v2"
    )
    assert "category" in (
        matching_store["predicate_indexes"]
    )


@pytest.mark.asyncio
async def test_store_and_search(
    backend: AIBackend,
    store_name: str,
) -> None:
    await backend.create_store(
        store_name=store_name,
        predicate_keys=["topic"],
        error_if_exists=True,
    )

    stored = await backend.store_entries(
        store_name=store_name,
        entries=[
            {
                "content": (
                    "Neural networks learn representations "
                    "from training data."
                ),
                "metadata": {
                    "topic": "machine-learning",
                },
            },
            {
                "content": (
                    "Tomatoes and basil are commonly used "
                    "in pasta sauce."
                ),
                "metadata": {
                    "topic": "cooking",
                },
            },
            {
                "content": (
                    "Jupiter is the largest planet in "
                    "the solar system."
                ),
                "metadata": {
                    "topic": "astronomy",
                },
            },
        ],
    )

    assert stored == {
        "inserted": 3,
        "updated": 0,
    }

    results = await backend.similarity_search(
        store_name=store_name,
        query="How do artificial neural networks learn?",
        top_k=3,
        algorithm="cosine",
        metadata_filter=None,
    )

    assert len(results) == 3
    assert (
        results[0]["metadata"]["topic"]
        == "machine-learning"
    )

@pytest.mark.asyncio
async def test_euclidean_search_preserves_server_order(
    backend: AIBackend,
    store_name: str,
) -> None:
    exact_content = (
        "Ahnlich performs semantic vector search."
    )

    await backend.create_store(
        store_name=store_name,
        predicate_keys=[],
        error_if_exists=True,
    )

    await backend.store_entries(
        store_name=store_name,
        entries=[
            {
                "content": exact_content,
                "metadata": {},
            },
            {
                "content": (
                    "Tomatoes are commonly used in pasta sauce."
                ),
                "metadata": {},
            },
            {
                "content": (
                    "Jupiter is the largest planet."
                ),
                "metadata": {},
            },
        ],
    )

    results = await backend.similarity_search(
        store_name=store_name,
        query=exact_content,
        top_k=3,
        algorithm="euclidean",
        metadata_filter=None,
    )

    distances = [
        result["similarity"]
        for result in results
    ]

    assert results[0]["content"] == exact_content
    assert distances == sorted(distances)


@pytest.mark.asyncio
async def test_store_entries_performs_upsert(
    backend: AIBackend,
    store_name: str,
) -> None:
    await backend.create_store(
        store_name=store_name,
        predicate_keys=["version"],
        error_if_exists=True,
    )

    inserted = await backend.store_entries(
        store_name=store_name,
        entries=[
            {
                "content": "A document that will be updated.",
                "metadata": {
                    "version": "one",
                },
            }
        ],
    )

    assert inserted == {
        "inserted": 1,
        "updated": 0,
    }

    updated = await backend.store_entries(
        store_name=store_name,
        entries=[
            {
                "content": "A document that will be updated.",
                "metadata": {
                    "version": "two",
                },
            }
        ],
    )

    assert updated == {
        "inserted": 0,
        "updated": 1,
    }

    version_two = await backend.get_by_metadata(
        store_name=store_name,
        metadata_filter={
            "version": "two",
        },
    )

    assert len(version_two) == 1
    assert (
        version_two[0]["metadata"]["version"]
        == "two"
    )


@pytest.mark.asyncio
async def test_filtered_similarity_search(
    backend: AIBackend,
    store_name: str,
) -> None:
    await backend.create_store(
        store_name=store_name,
        predicate_keys=["category"],
        error_if_exists=True,
    )

    await backend.store_entries(
        store_name=store_name,
        entries=[
            {
                "content": "A guide to Python programming.",
                "metadata": {
                    "category": "code",
                },
            },
            {
                "content": "A guide to Italian cooking.",
                "metadata": {
                    "category": "food",
                },
            },
            {
                "content": "Python type checking with mypy.",
                "metadata": {
                    "category": "code",
                },
            },
        ],
    )

    results = await backend.similarity_search(
        store_name=store_name,
        query="help with programming",
        top_k=5,
        algorithm="cosine",
        metadata_filter={
            "category": "code",
        },
    )

    assert len(results) == 2
    assert all(
        result["metadata"]["category"] == "code"
        for result in results
    )


@pytest.mark.asyncio
async def test_predicate_indexes(
    backend: AIBackend,
    store_name: str,
) -> None:
    await backend.create_store(
        store_name=store_name,
        predicate_keys=[],
        error_if_exists=True,
    )

    created = await backend.create_predicate_index(
        store_name=store_name,
        keys=["author"],
    )

    assert created == 1

    dropped = await backend.drop_predicate_index(
        store_name=store_name,
        keys=["author"],
    )

    assert dropped == 1


@pytest.mark.asyncio
async def test_delete_by_metadata(
    backend: AIBackend,
    store_name: str,
) -> None:
    await backend.create_store(
        store_name=store_name,
        predicate_keys=["status"],
        error_if_exists=True,
    )

    await backend.store_entries(
        store_name=store_name,
        entries=[
            {
                "content": "Keep this document.",
                "metadata": {
                    "status": "active",
                },
            },
            {
                "content": "Delete this document.",
                "metadata": {
                    "status": "archived",
                },
            },
            {
                "content": "Delete this old note too.",
                "metadata": {
                    "status": "archived",
                },
            },
        ],
    )

    deleted = await backend.delete_by_metadata(
        store_name=store_name,
        metadata_filter={
            "status": "archived",
        },
    )

    assert deleted == 2

    archived = await backend.get_by_metadata(
        store_name=store_name,
        metadata_filter={
            "status": "archived",
        },
    )

    assert archived == []

    active = await backend.get_by_metadata(
        store_name=store_name,
        metadata_filter={
            "status": "active",
        },
    )

    assert len(active) == 1