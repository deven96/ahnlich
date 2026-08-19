from __future__ import annotations

from ahnlich_mcp.backends.ai import AIBackend, TextPreprocessing
from ahnlich_mcp.backends.base import (
    AhnlichConnectionError,
    AhnlichError,
    PredicateIndexNotFoundError,
    StoreNotFoundError,
)
from ahnlich_mcp.backends.db import DBBackend
from ahnlich_mcp.config import Profile, Settings

Backend = AIBackend | DBBackend


def create_backend(settings: Settings) -> Backend:
    if settings.profile is Profile.DB:
        return DBBackend(
            host=settings.host,
            port=settings.port,
        )

    if settings.ai_model is None:
        raise ValueError("AI profile requires an AI model")

    return AIBackend(
        host=settings.host,
        port=settings.port,
        model=settings.ai_model,
    )


__all__ = [
    "AIBackend",
    "AhnlichConnectionError",
    "AhnlichError",
    "Backend",
    "DBBackend",
    "PredicateIndexNotFoundError",
    "StoreNotFoundError",
    "create_backend",
    "TextPreprocessing",
]