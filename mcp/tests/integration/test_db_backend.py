from __future__ import annotations

from typing import Any
from uuid import uuid4

import pytest
import pytest_asyncio

from ahnlich_mcp.backends.db import DBBackend
from ahnlich_mcp.backends.base import PredicateIndexNotFoundError

pytestmark = pytest.mark.integration


def unique_store_name(prefix: str) -> str:
    return f"{prefix}_{uuid4().hex}"


@pytest_asyncio.fixture
async def backend() -> DBBackend:
    db_backend = DBBackend(
        host="127.0.0.1",
        port=1369,
    )

    yield db_backend

    await db_backend.close()


@pytest_asyncio.fixture
async def store_name(
    backend: DBBackend,
) -> str:
    name = unique_store_name("db_backend_test")

    yield name

    await backend.drop_store(
        store_name=name,
        error_if_not_exists=False,
    )


def test_backend_configuration() -> None:
    backend = DBBackend(
        host="db.internal",
        port=8000,
    )

    assert backend.profile_name == "db"
    assert backend.service_name == "ahnlich-db"
    assert backend.endpoint == "db.internal:8000"


@pytest.mark.parametrize(
    "dimension",
    [1, 3, 384],
)
def test_valid_dimension(
    dimension: int,
) -> None:
    assert (
        DBBackend._validate_dimension(dimension)
        == dimension
    )


@pytest.mark.parametrize(
    "dimension",
    [0, -1, True, 3.5, "384"],
)
def test_invalid_dimension(
    dimension: Any,
) -> None:
    with pytest.raises(ValueError):
        DBBackend._validate_dimension(dimension)


def test_build_entries() -> None:
    backend = DBBackend(
        host="127.0.0.1",
        port=1369,
    )

    entries = backend._build_entries(
        [
            {
                "embedding": [1, 0.5, -1],
                "metadata": {
                    "topic": "example",
                },
            }
        ]
    )

    assert len(entries) == 1
    assert list(entries[0].key.key) == [
        1.0,
        0.5,
        -1.0,
    ]
    assert (
        entries[0]
        .value
        .value["topic"]
        .raw_string
        == "example"
    )


@pytest.mark.parametrize(
    "entries",
    [
        [],
        ["not-a-dictionary"],
        [{}],
        [{"embedding": []}],
        [{"embedding": "not-a-list"}],
        [
            {
                "embedding": [1, 2, 3],
                "metadata": "not-a-dictionary",
            }
        ],
    ],
)
def test_invalid_entries(
    entries: list[Any],
) -> None:
    backend = DBBackend(
        host="127.0.0.1",
        port=1369,
    )

    with pytest.raises(ValueError):
        backend._build_entries(entries)


@pytest.mark.asyncio
async def test_ping(
    backend: DBBackend,
) -> None:
    assert await backend.ping() is True


@pytest.mark.asyncio
async def test_server_info(
    backend: DBBackend,
) -> None:
    result = await backend.server_info()

    assert result["profile"] == "db"
    assert result["service"] == "ahnlich-db"
    assert result["version"]


@pytest.mark.asyncio
async def test_create_and_list_store(
    backend: DBBackend,
    store_name: str,
) -> None:
    await backend.create_store(
        store_name=store_name,
        dimension=3,
        predicate_keys=["category"],
        error_if_exists=True,
    )

    stores = await backend.list_stores()

    matching_store = next(
        store
        for store in stores
        if store["name"] == store_name
    )

    assert matching_store["dimension"] == 3
    assert "category" in (
        matching_store["predicate_indexes"]
    )
    assert matching_store["entry_count"] == 0


@pytest.mark.asyncio
async def test_store_and_search(
    backend: DBBackend,
    store_name: str,
) -> None:
    await backend.create_store(
        store_name=store_name,
        dimension=3,
        predicate_keys=["topic"],
        error_if_exists=True,
    )

    stored = await backend.store_entries(
        store_name=store_name,
        entries=[
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
            {
                "embedding": [0.0, 0.0, 1.0],
                "metadata": {
                    "topic": "blue",
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
        query_embedding=[0.9, 0.1, 0.0],
        top_k=3,
        algorithm="cosine",
        metadata_filter=None,
    )

    assert len(results) == 3
    assert results[0]["metadata"]["topic"] == "red"


@pytest.mark.asyncio
async def test_store_entries_performs_upsert(
    backend: DBBackend,
    store_name: str,
) -> None:
    await backend.create_store(
        store_name=store_name,
        dimension=3,
        predicate_keys=["version"],
        error_if_exists=True,
    )

    inserted = await backend.store_entries(
        store_name=store_name,
        entries=[
            {
                "embedding": [1.0, 0.0, 0.0],
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
                "embedding": [1.0, 0.0, 0.0],
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

    entries = await backend.get_by_metadata(
        store_name=store_name,
        metadata_filter={
            "version": "two",
        },
    )

    assert len(entries) == 1
    assert entries[0]["metadata"]["version"] == "two"


@pytest.mark.asyncio
async def test_filtered_search(
    backend: DBBackend,
    store_name: str,
) -> None:
    await backend.create_store(
        store_name=store_name,
        dimension=3,
        predicate_keys=["category"],
        error_if_exists=True,
    )

    await backend.store_entries(
        store_name=store_name,
        entries=[
            {
                "embedding": [1.0, 0.0, 0.0],
                "metadata": {
                    "category": "included",
                },
            },
            {
                "embedding": [0.9, 0.1, 0.0],
                "metadata": {
                    "category": "excluded",
                },
            },
            {
                "embedding": [0.8, 0.2, 0.0],
                "metadata": {
                    "category": "included",
                },
            },
        ],
    )

    results = await backend.similarity_search(
        store_name=store_name,
        query_embedding=[1.0, 0.0, 0.0],
        top_k=5,
        algorithm="cosine",
        metadata_filter={
            "category": "included",
        },
    )

    assert len(results) == 2
    assert all(
        result["metadata"]["category"]
        == "included"
        for result in results
    )


@pytest.mark.asyncio
async def test_predicate_indexes(
    backend: DBBackend,
    store_name: str,
) -> None:
    await backend.create_store(
        store_name=store_name,
        dimension=3,
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
    backend: DBBackend,
    store_name: str,
) -> None:
    await backend.create_store(
        store_name=store_name,
        dimension=3,
        predicate_keys=["status"],
        error_if_exists=True,
    )

    await backend.store_entries(
        store_name=store_name,
        entries=[
            {
                "embedding": [1.0, 0.0, 0.0],
                "metadata": {
                    "status": "active",
                },
            },
            {
                "embedding": [0.0, 1.0, 0.0],
                "metadata": {
                    "status": "archived",
                },
            },
            {
                "embedding": [0.0, 0.0, 1.0],
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

@pytest.mark.asyncio
async def test_missing_predicate_index_message_is_classified(
    backend: DBBackend,
    store_name: str,
) -> None:
    await backend.create_store(
        store_name=store_name,
        dimension=3,
        predicate_keys=[],
        error_if_exists=True,
    )

    with pytest.raises(
        PredicateIndexNotFoundError,
    ) as raised:
        await backend.drop_predicate_index(
            store_name=store_name,
            keys=["category"],
            error_if_not_exists=True,
        )

    assert str(raised.value) == (
        "Predicate category not found in store, "
        "attempt CREATEPREDINDEX with predicate"
    )
