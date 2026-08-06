from __future__ import annotations

import math
from typing import Any

import pytest

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