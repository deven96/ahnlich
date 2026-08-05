from __future__ import annotations

from ahnlich_mcp.backends.base import (
    AhnlichConnectionError,
    PredicateIndexNotFoundError,
    StoreNotFoundError,
)
from ahnlich_mcp.backends.db import DBBackend
from ahnlich_mcp.tools import error_response


def backend() -> DBBackend:
    return DBBackend(
        host="127.0.0.1",
        port=1369,
    )


def test_connection_error_contains_recovery_information() -> None:
    response = error_response(
        AhnlichConnectionError(
            service_name="ahnlich-db",
            host="127.0.0.1",
            port=1369,
            detail="connection refused",
        ),
        backend=backend(),
    )

    assert response["status"] == "error"
    assert response["profile"] == "db"
    assert response["endpoint"] == "127.0.0.1:1369"
    assert "Start ahnlich-db" in response["suggested_action"]


def test_missing_store_error_suggests_create_store() -> None:
    response = error_response(
        StoreNotFoundError("documents"),
        backend=backend(),
    )

    assert response["status"] == "error"
    assert "documents" in response["error"]
    assert "create_store" in response["suggested_action"]


def test_predicate_error_suggests_creating_index() -> None:
    response = error_response(
        PredicateIndexNotFoundError(
            "Predicate index not found"
        ),
        backend=backend(),
    )

    assert response["status"] == "error"
    assert (
        "create_predicate_index"
        in response["suggested_action"]
    )


def test_unknown_error_is_safely_serialized() -> None:
    response = error_response(
        RuntimeError("unexpected failure"),
        backend=backend(),
    )

    assert response == {
        "status": "error",
        "error": "unexpected failure",
    }