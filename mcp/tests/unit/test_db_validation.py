from __future__ import annotations

import math
from typing import Any

import pytest
from types import SimpleNamespace
from ahnlich_mcp.backends.db import DBBackend


@pytest.fixture
def backend() -> DBBackend:
    return DBBackend(
        host="127.0.0.1",
        port=1369,
    )


def test_embedding_is_normalized_to_floats(
    backend: DBBackend,
) -> None:
    result = backend._validate_embedding(
        [1, 2.5, -3]
    )

    assert result == [1.0, 2.5, -3.0]
    assert all(
        isinstance(value, float)
        for value in result
    )


@pytest.mark.parametrize(
    "embedding",
    [
        "not-a-list",
        [],
        [True, 0.2],
        ["invalid", 0.2],
        [math.nan, 0.2],
        [math.inf, 0.2],
        [-math.inf, 0.2],
    ],
)
def test_invalid_embedding(
    backend: DBBackend,
    embedding: Any,
) -> None:
    with pytest.raises(ValueError):
        backend._validate_embedding(embedding)

def test_entry_identity_omits_embedding_by_default(
    backend: DBBackend,
) -> None:
    entry = SimpleNamespace(
        key=SimpleNamespace(
            key=[0.1, 0.2, 0.3],
        )
    )

    assert backend._entry_identity(entry) == {
        "dimension": 3,
    }


def test_entry_identity_can_include_embedding(
    backend: DBBackend,
) -> None:
    entry = SimpleNamespace(
        key=SimpleNamespace(
            key=[0.1, 0.2, 0.3],
        )
    )

    assert backend._entry_identity(
        entry,
        include_embeddings=True,
    ) == {
        "embedding": [0.1, 0.2, 0.3],
    }