from __future__ import annotations

import math
from typing import Any

import pytest
from grpclib.client import Channel

from ahnlich_client_py.grpc.algorithm.algorithms import (
    Algorithm,
)
from ahnlich_client_py.grpc.predicates import (
    PredicateCondition,
)

from ahnlich_mcp.backends.base import (
    AhnlichConnectionError,
    BaseBackend,
)


class ExampleBackend(BaseBackend):
    """
    Minimal concrete backend used to test shared behavior.

    It does not make network calls.
    """

    def _create_stub(self, channel: Channel) -> Any:
        return object()


@pytest.fixture
def backend() -> ExampleBackend:
    return ExampleBackend(
        service_name="example-service",
        host="127.0.0.1",
        port=9999,
    )


def test_endpoint(backend: ExampleBackend) -> None:
    assert backend.endpoint == "127.0.0.1:9999"


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
    [1, 5, 100],
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


def test_embedding_is_normalized_to_floats(
    backend: ExampleBackend,
) -> None:
    result = backend._validate_embedding(
        [1, 2.5, -3],
    )

    assert result == [1.0, 2.5, -3.0]
    assert all(
        isinstance(value, float)
        for value in result
    )


@pytest.mark.parametrize(
    "embedding",
    [
        [],
        [True, 0.2],
        ["invalid", 0.2],
        [math.nan, 0.2],
        [math.inf, 0.2],
        [-math.inf, 0.2],
    ],
)
def test_invalid_embedding(
    backend: ExampleBackend,
    embedding: list[Any],
) -> None:
    with pytest.raises(ValueError):
        backend._validate_embedding(embedding)


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