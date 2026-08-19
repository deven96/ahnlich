from __future__ import annotations

import pytest

from ahnlich_mcp.backends import (
    AIBackend,
    DBBackend,
    create_backend,
)
from ahnlich_mcp.models import (
    TEXT_MODEL_NAMES,
    TEXT_MODELS,
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

@pytest.mark.parametrize(
    "model_name",
    tuple(TEXT_MODELS),
)
def test_ai_backend_supports_text_models(
    model_name: str,
) -> None:
    backend = AIBackend(
        host="ai.internal",
        port=1370,
        model=model_name,
    )

    assert backend.model_name == model_name
    assert (
        backend.model_value
        == TEXT_MODELS[model_name]
    )


def test_text_model_names_round_trip() -> None:
    for name, model in TEXT_MODELS.items():
        assert TEXT_MODEL_NAMES[model] == name


def test_ai_backend_rejects_unsupported_model() -> None:
    with pytest.raises(
        ValueError,
        match="Unsupported AI model",
    ):
        AIBackend(
            host="ai.internal",
            port=1370,
            model="unknown-model",
        )