from __future__ import annotations

import os
from dataclasses import dataclass
from enum import Enum

from ahnlich_mcp.models import (
    DEFAULT_TEXT_MODEL,
    TEXT_MODELS,
)


class ConfigurationError(ValueError):
    pass

class Profile(str, Enum):
    DB = "db"
    AI = "ai"

@dataclass(frozen=True)
class Settings:
    profile: Profile
    host: str
    port: int
    ai_model: str | None = None

    @classmethod
    def from_env(
        cls,
        profile: str | Profile | None = None,
    ) -> Settings:
        selected_profile = cls._resolve_profile(profile)

        if selected_profile is Profile.DB:
            return cls(
                profile=selected_profile,
                host=cls._read_host(
                    "AHNLICH_DB_HOST",
                    "127.0.0.1",
                ),
                port=cls._read_port(
                    "AHNLICH_DB_PORT",
                    1369,
                ),
            )

        model = cls._read_model()

        return cls(
            profile=selected_profile,
            host=cls._read_host(
                "AHNLICH_AI_HOST",
                "127.0.0.1",
            ),
            port=cls._read_port(
                "AHNLICH_AI_PORT",
                1370,
            ),
            ai_model=model,
        )

    @staticmethod
    def _resolve_profile(
        profile: str | Profile | None,
    ) -> Profile:
        if isinstance(profile, Profile):
            return profile

        value = profile or os.getenv(
            "AHNLICH_PROFILE",
            Profile.AI.value,
        )

        try:
            return Profile(value.strip().lower())
        except ValueError as error:
            supported = ", ".join(
                profile.value for profile in Profile
            )
            raise ConfigurationError(
                f"Invalid Ahnlich profile {value!r}. "
                f"Supported profiles: {supported}"
            ) from error

    @staticmethod
    def _read_model() -> str:
        model = os.getenv(
            "AHNLICH_AI_MODEL",
            DEFAULT_TEXT_MODEL,
        ).strip()

        if not model:
            raise ConfigurationError("AHNLICH_AI_MODEL cannot be empty")

        if model not in TEXT_MODELS:
            supported = ", ".join(sorted(TEXT_MODELS))
            raise ConfigurationError(
                f"Unsupported AI model {model!r}."
                f"Supported models: {supported}"
            )
        
        return model

    @staticmethod
    def _read_host(variable: str, default: str) -> str:
        host = os.getenv(variable, default).strip()

        if not host:
            raise ConfigurationError(f"{variable} cannot be empty")

        return host

    @staticmethod
    def _read_port(variable: str, default: int) -> int:
        raw_value = os.getenv(variable, str(default))

        try:
            port = int(raw_value)
        except ValueError as error:
            raise ConfigurationError(
                f"{variable} must be an integer"
            ) from error

        if not 1 <= port <= 65535:
            raise ConfigurationError(
                f"{variable} must be between 1 and 65535"
            )

        return port