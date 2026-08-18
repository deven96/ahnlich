from __future__ import annotations

import json
from unittest.mock import AsyncMock

import pytest
from mcp.server.fastmcp.exceptions import ToolError

from ahnlich_mcp.backends.base import (
    AhnlichConnectionError,
    PredicateIndexNotFoundError,
    StoreNotFoundError,
)
from ahnlich_mcp.backends.db import DBBackend
from ahnlich_mcp.tools import (
    error_payload,
    raise_tool_error,
)


def backend() -> DBBackend:
    return DBBackend(
        host="127.0.0.1",
        port=1369,
    )


def test_connection_error_contains_recovery_information() -> None:
    payload = error_payload(
        AhnlichConnectionError(
            service_name="ahnlich-db",
            host="127.0.0.1",
            port=1369,
            detail="connection refused",
        ),
        backend=backend(),
    )

    assert payload["status"] == "error"
    assert payload["profile"] == "db"
    assert payload["endpoint"] == "127.0.0.1:1369"
    assert "Start ahnlich-db" in payload["suggested_action"]


def test_missing_store_error_suggests_create_store() -> None:
    payload = error_payload(
        StoreNotFoundError("documents"),
        backend=backend(),
    )

    assert payload["status"] == "error"
    assert "documents" in payload["error"]
    assert "create_store" in payload["suggested_action"]


def test_predicate_error_suggests_creating_index() -> None:
    payload = error_payload(
        PredicateIndexNotFoundError(
            "Predicate index not found"
        ),
        backend=backend(),
    )

    assert payload["status"] == "error"
    assert (
        "create_predicate_index"
        in payload["suggested_action"]
    )


def test_unknown_error_is_safely_serialized() -> None:
    payload = error_payload(
        RuntimeError("unexpected failure"),
        backend=backend(),
    )

    assert payload == {
        "status": "error",
        "error": "unexpected failure",
    }


def test_raise_tool_error_preserves_json_payload() -> None:
    with pytest.raises(ToolError) as raised:
        raise_tool_error(
            StoreNotFoundError("documents"),
            backend=backend(),
        )

    payload = json.loads(
        str(raised.value)
    )

    assert payload["status"] == "error"
    assert "documents" in payload["error"]
    assert "create_store" in payload["suggested_action"]