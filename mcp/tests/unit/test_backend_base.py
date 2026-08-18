from __future__ import annotations


from typing import Any

import pytest
from types import SimpleNamespace
from unittest.mock import AsyncMock, Mock

from ahnlich_client_py.grpc.algorithm.algorithms import (
    Algorithm,
)
from ahnlich_client_py.grpc.predicates import (
    PredicateCondition,
)

from ahnlich_mcp.backends.base import (
    AhnlichConnectionError,
    BaseBackend,
    AhnlichError,
    MAX_TOP_K
)
from ahnlich_mcp.backends.ai import AIBackend

class ExampleRequest:
    def __init__(
        self,
        **fields: Any,
    ) -> None:
        self.__dict__.update(fields)


class ExampleQueries:
    Ping = ExampleRequest
    InfoServer = ExampleRequest
    DropStore = ExampleRequest
    DelPred = ExampleRequest
    CreatePredIndex = ExampleRequest
    DropPredIndex = ExampleRequest
    GetPred = ExampleRequest
    ListStores = ExampleRequest


class ExampleStub:
    def __init__(self, channel: object) -> None:
        self.channel = channel


class ExampleBackend(BaseBackend):
    profile_name = "example"
    service_name = "example-service"
    query_module = ExampleQueries
    stub_type = ExampleStub

    def _entry_identity(
        self,
        entry: Any,
        *,
        include_embeddings: bool = False,
    ) -> dict[str, Any]:
        return {
            "key": entry.key,
        }

    def _build_set_request(
        self, *,
        store_name: str,
        entries: list[dict[str, Any]],
    ) -> ExampleRequest:
        return ExampleRequest(
            store=store_name,
            inputs=entries,
        )

    def _format_store(self, store: Any) -> dict[str, Any]:
        return {
            "name": store.name,
        }


@pytest.fixture
def backend() -> ExampleBackend:
    return ExampleBackend(
        host="127.0.0.1",
        port=9999,
    )


def test_endpoint(backend: ExampleBackend) -> None:
    assert backend.endpoint == "127.0.0.1:9999"

@pytest.mark.asyncio
async def test_shared_ping(
    backend: ExampleBackend,
) -> None:
    backend._call = AsyncMock(
        return_value=object()
    )

    available = await backend.ping()

    assert available is True
    backend._call.assert_awaited_once()

    method_name, request = (
        backend._call.await_args.args
    )

    assert method_name == "ping"
    assert isinstance(
        request,
        ExampleQueries.Ping,
    )


@pytest.mark.asyncio
async def test_shared_ping_returns_false_on_error(
    backend: ExampleBackend,
) -> None:
    backend._call = AsyncMock(
        side_effect=AhnlichError("unavailable")
    )

    assert await backend.ping() is False


@pytest.mark.asyncio
async def test_shared_list_stores(
    backend: ExampleBackend,
) -> None:
    backend._call = AsyncMock(
        return_value=SimpleNamespace(
            stores=[
                SimpleNamespace(name="documents"),
                SimpleNamespace(name="notes"),
            ]
        )
    )

    stores = await backend.list_stores()

    assert stores == [
        {"name": "documents"},
        {"name": "notes"},
    ]

    method_name, request = (
        backend._call.await_args.args
    )

    assert method_name == "list_stores"
    assert isinstance(
        request,
        ExampleQueries.ListStores,
    )


@pytest.mark.asyncio
async def test_similarity_search_preserves_server_order(backend: ExampleBackend) -> None:
    backend._call = AsyncMock(
        return_value=SimpleNamespace(
            entries=[
                SimpleNamespace(
                    key="nearest",
                    value=backend._serialize_metadata({}),
                    similarity=SimpleNamespace(
                        value=0.1
                    ),
                ),
                SimpleNamespace(
                    key="farthest",
                    value=backend._serialize_metadata({}),
                    similarity=SimpleNamespace(
                        value=0.8
                    ),
                ),
            ]
        )
    )
    request = ExampleRequest(
        store="documents"
    )

    results = await backend._execute_similarity_search(
        request=request,
        store_name="documents",
        metadata_filter_applied=True,
    )

    assert [
        result["key"]
        for result in results
    ] == ["nearest", "farthest"]

    backend._call.assert_awaited_once_with(
        "get_sim_n",
        request,
        store_name="documents",
        predicate_operation=True,
    )

@pytest.mark.asyncio
async def test_ai_search_builds_metadata_condition_once() -> None:
    backend = AIBackend(
        host="127.0.0.1",
        port=1370,
        model="all-minilm-l6-v2",
    )
    build_condition = Mock(
        wraps=backend._build_condition
    )
    backend._build_condition = build_condition
    backend._execute_similarity_search = AsyncMock(
        return_value=[]
    )

    await backend.similarity_search(
        store_name="documents",
        query="vector databases",
        top_k=5,
        algorithm="cosine",
        metadata_filter={
            "category": "documentation",
        },
    )

    build_condition.assert_called_once_with(
        {
            "category": "documentation",
        }
    )

@pytest.mark.asyncio
async def test_shared_server_info(
    backend: ExampleBackend,
) -> None:
    backend._call = AsyncMock(
        return_value=SimpleNamespace(
            info=SimpleNamespace(
                address="127.0.0.1:9999",
                version="1.0.0",
                type="Example",
                limit=100,
                remaining=90,
            )
        )
    )

    result = await backend.server_info()

    assert result == {
        "profile": "example",
        "service": "example-service",
        "address": "127.0.0.1:9999",
        "version": "1.0.0",
        "type": "example",
        "limit": 100,
        "remaining": 90,
    }

def test_shared_entry_formatting(
    backend: ExampleBackend,
) -> None:
    entry = SimpleNamespace(
        key="document-1",
        value=backend._serialize_metadata(
            {
                "topic": "testing",
            }
        ),
    )

    assert backend._format_entry(entry) == {
        "key": "document-1",
        "metadata": {
            "topic": "testing",
        },
    }

@pytest.mark.asyncio
async def test_shared_store_entries(backend):
    backend._call = AsyncMock(
        return_value=SimpleNamespace(
            upsert=SimpleNamespace(
                inserted=2,
                updated=1,
            )
        )
    )
    entries = [
        {"key": "first"},
        {"key": "second"},
    ]

    result = await backend.store_entries(
        store_name="documents",
        entries=entries,
    )

    assert result == {
        "inserted": 2,
        "updated": 1,
    }

    method_name, request = backend._call.await_args.args
    kwargs = backend._call.await_args.kwargs

    assert method_name == "set"
    assert request.store == "documents"
    assert request.inputs == entries
    assert kwargs == {"store_name": "documents"}

@pytest.mark.asyncio
async def test_shared_get_by_metadata(
    backend: ExampleBackend,
) -> None:
    backend._call = AsyncMock(
        return_value=SimpleNamespace(
            entries=[
                SimpleNamespace(
                    key="document-1",
                    value=backend._serialize_metadata(
                        {
                            "status": "active",
                        }
                    ),
                )
            ]
        )
    )

    entries = await backend.get_by_metadata(
        store_name="documents",
        metadata_filter={
            "status": "active",
        },
    )

    assert entries == [
        {
            "key": "document-1",
            "metadata": {
                "status": "active",
            },
        }
    ]

    method_name, request = (
        backend._call.await_args.args
    )
    keyword_arguments = (
        backend._call.await_args.kwargs
    )

    assert method_name == "get_pred"
    assert request.store == "documents"
    assert isinstance(
        request.condition,
        PredicateCondition,
    )
    assert keyword_arguments == {
        "store_name": "documents",
        "predicate_operation": True,
    }

def test_shared_search_entry_formatting(
    backend: ExampleBackend,
) -> None:
    entry = SimpleNamespace(
        key="document-1",
        value=backend._serialize_metadata(
            {
                "topic": "testing",
            }
        ),
        similarity=SimpleNamespace(
            value=0.75
        ),
    )

    assert backend._format_search_entries(
        [entry]
    ) == [
        {
            "key": "document-1",
            "metadata": {
                "topic": "testing",
            },
            "similarity": 0.75,
        }
    ]

@pytest.mark.asyncio
async def test_shared_drop_store(
    backend: ExampleBackend,
) -> None:
    backend._call = AsyncMock(
        return_value=SimpleNamespace(
            deleted_count=1
        )
    )

    deleted = await backend.drop_store(
        store_name="documents",
        error_if_not_exists=True,
    )

    assert deleted == 1

    method_name, request = (
        backend._call.await_args.args
    )
    keyword_arguments = (
        backend._call.await_args.kwargs
    )

    assert method_name == "drop_store"
    assert request.store == "documents"
    assert request.error_if_not_exists is True
    assert keyword_arguments == {
        "store_name": "documents",
    }


@pytest.mark.asyncio
async def test_shared_delete_by_metadata(
    backend: ExampleBackend,
) -> None:
    backend._call = AsyncMock(
        return_value=SimpleNamespace(
            deleted_count=2
        )
    )

    deleted = await backend.delete_by_metadata(
        store_name="documents",
        metadata_filter={
            "status": "archived",
        },
    )

    assert deleted == 2

    method_name, request = (
        backend._call.await_args.args
    )
    keyword_arguments = (
        backend._call.await_args.kwargs
    )

    assert method_name == "del_pred"
    assert request.store == "documents"
    assert isinstance(
        request.condition,
        PredicateCondition,
    )
    assert keyword_arguments == {
        "store_name": "documents",
        "predicate_operation": True,
    }


@pytest.mark.asyncio
async def test_shared_create_predicate_index(
    backend: ExampleBackend,
) -> None:
    backend._call = AsyncMock(
        return_value=SimpleNamespace(
            created_indexes=2
        )
    )

    created = await backend.create_predicate_index(
        store_name="documents",
        keys=["status", "category"],
    )

    assert created == 2

    method_name, request = (
        backend._call.await_args.args
    )
    keyword_arguments = (
        backend._call.await_args.kwargs
    )

    assert method_name == "create_pred_index"
    assert request.store == "documents"
    assert request.predicates == [
        "status",
        "category",
    ]
    assert keyword_arguments == {
        "store_name": "documents",
        "predicate_operation": True,
    }


@pytest.mark.asyncio
async def test_shared_drop_predicate_index(
    backend: ExampleBackend,
) -> None:
    backend._call = AsyncMock(
        return_value=SimpleNamespace(
            deleted_count=2
        )
    )

    deleted = await backend.drop_predicate_index(
        store_name="documents",
        keys=["status", "category"],
        error_if_not_exists=False,
    )

    assert deleted == 2

    method_name, request = (
        backend._call.await_args.args
    )
    keyword_arguments = (
        backend._call.await_args.kwargs
    )

    assert method_name == "drop_pred_index"
    assert request.store == "documents"
    assert request.predicates == [
        "status",
        "category",
    ]
    assert request.error_if_not_exists is False
    assert keyword_arguments == {
        "store_name": "documents",
        "predicate_operation": True,
    }

def test_connection_error_contains_endpoint() -> None:
    error = AhnlichConnectionError(
        service_name="ahnlich-db",
        host="db.internal",
        port=1369,
        detail="connection refused",
    )

    assert error.service_name == "ahnlich-db"
    assert error.host == "db.internal"
    assert error.port == 1369
    assert error.detail == "connection refused"
    assert str(error) == (
        "Cannot connect to ahnlich-db at "
        "db.internal:1369: connection refused"
    )


@pytest.mark.parametrize(
    "algorithm, expected",
    [
        (
            "cosine",
            Algorithm.CosineSimilarity,
        ),
        (
            "euclidean",
            Algorithm.EuclideanDistance,
        ),
        (
            "dot_product",
            Algorithm.DotProductSimilarity,
        ),
    ],
)
def test_resolve_algorithm(
    backend: ExampleBackend,
    algorithm: str,
    expected: Algorithm,
) -> None:
    assert backend._resolve_algorithm(algorithm) == expected


def test_unsupported_algorithm(
    backend: ExampleBackend,
) -> None:
    with pytest.raises(
        ValueError,
        match="Unsupported algorithm",
    ):
        backend._resolve_algorithm("manhattan")


@pytest.mark.parametrize(
    "top_k",
    [1, 5, 100, MAX_TOP_K],
)
def test_valid_top_k(
    backend: ExampleBackend,
    top_k: int,
) -> None:
    assert backend._validate_top_k(top_k) == top_k


@pytest.mark.parametrize(
    "top_k",
    [0, -1],
)
def test_invalid_top_k_range(
    backend: ExampleBackend,
    top_k: int,
) -> None:
    with pytest.raises(
        ValueError,
        match="at least 1",
    ):
        backend._validate_top_k(top_k)

def test_top_k_above_maximum_is_rejected(
    backend: ExampleBackend,
) -> None:
    with pytest.raises(
        ValueError,
        match=f"must not exceed {MAX_TOP_K}",
    ):
        backend._validate_top_k(MAX_TOP_K + 1)

@pytest.mark.parametrize(
    "top_k",
    [True, 2.5, "5"],
)
def test_invalid_top_k_type(
    backend: ExampleBackend,
    top_k: Any,
) -> None:
    with pytest.raises(
        ValueError,
        match="must be an integer",
    ):
        backend._validate_top_k(top_k)

def test_metadata_round_trip(
    backend: ExampleBackend,
) -> None:
    metadata = {
        "filename": "report.pdf",
        "directory": "Downloads",
    }

    serialized = backend._serialize_metadata(metadata)
    deserialized = backend._deserialize_metadata(serialized)

    assert deserialized == metadata


@pytest.mark.parametrize(
    "metadata",
    [
        {"valid": 123},
        {"": "value"},
        {1: "value"},
    ],
)
def test_invalid_metadata(
    backend: ExampleBackend,
    metadata: dict[Any, Any],
) -> None:
    with pytest.raises(ValueError):
        backend._serialize_metadata(metadata)


def test_single_metadata_condition(
    backend: ExampleBackend,
) -> None:
    condition = backend._build_condition(
        {"extension": ".pdf"}
    )

    assert isinstance(
        condition,
        PredicateCondition,
    )


def test_multiple_metadata_conditions(
    backend: ExampleBackend,
) -> None:
    condition = backend._build_condition(
        {
            "extension": ".pdf",
            "directory": "Downloads",
        }
    )

    assert isinstance(
        condition,
        PredicateCondition,
    )

def test_empty_predicate_keys_can_be_allowed(
    backend: ExampleBackend,
) -> None:
    assert backend._validate_predicate_keys(
        [],
        allow_empty=True,
    ) == []


@pytest.mark.parametrize(
    "metadata_filter",
    [
        {},
        {"": "value"},
        {"valid": 123},
    ],
)
def test_invalid_metadata_filter(
    backend: ExampleBackend,
    metadata_filter: dict[Any, Any],
) -> None:
    with pytest.raises(ValueError):
        backend._build_condition(metadata_filter)


def test_predicate_keys(
    backend: ExampleBackend,
) -> None:
    result = backend._validate_predicate_keys(
        ["category", "directory"]
    )

    assert result == ["category", "directory"]


@pytest.mark.parametrize(
    "keys",
    [
        [],
        [""],
        ["category", "category"],
        ["category", 123],
    ],
)
def test_invalid_predicate_keys(
    backend: ExampleBackend,
    keys: list[Any],
) -> None:
    with pytest.raises(ValueError):
        backend._validate_predicate_keys(keys)


@pytest.mark.parametrize(
    "store_name",
    [
        "documents",
        "my_vectors",
        "project-knowledge",
    ],
)
def test_valid_store_name(
    backend: ExampleBackend,
    store_name: str,
) -> None:
    assert (
        backend._validate_store_name(store_name)
        == store_name
    )


@pytest.mark.parametrize(
    "store_name",
    [
        "",
        "   ",
    ],
)
def test_empty_store_name(
    backend: ExampleBackend,
    store_name: str,
) -> None:
    with pytest.raises(
        ValueError,
        match="must not be empty",
    ):
        backend._validate_store_name(store_name)