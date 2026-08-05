from __future__ import annotations

import pytest

from ahnlich_mcp.backends import (
    AIBackend,
    DBBackend,
    create_backend,
)
from ahnlich_mcp.config import Profile, Settings


def test_create_db_backend() -> None:
    settings = Settings(
        profile=Profile.DB,
        host="db.internal",
        port=1369,
    )

    backend = create_backend(settings)

    assert isinstance(backend, DBBackend)
    assert backend.endpoint == "db.internal:1369"


def test_create_ai_backend() -> None:
    settings = Settings(
        profile=Profile.AI,
        host="ai.internal",
        port=1370,
        ai_model="all-minilm-l6-v2",
    )

    backend = create_backend(settings)

    assert isinstance(backend, AIBackend)
    assert backend.endpoint == "ai.internal:1370"
    assert backend.model_name == "all-minilm-l6-v2"


def test_ai_backend_requires_model() -> None:
    settings = Settings(
        profile=Profile.AI,
        host="ai.internal",
        port=1370,
        ai_model=None,
    )

    with pytest.raises(
        ValueError,
        match="AI profile requires an AI model",
    ):
        create_backend(settings)