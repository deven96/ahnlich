from __future__ import annotations

import pytest

from ahnlich_mcp.config import (
    ConfigurationError,
    Profile,
    Settings,
)
from ahnlich_mcp.models import (
    DEFAULT_TEXT_MODEL,
    TEXT_MODELS,
)

AHNLICH_ENVIRONMENT_VARIABLES = (
    "AHNLICH_PROFILE",
    "AHNLICH_DB_HOST",
    "AHNLICH_DB_PORT",
    "AHNLICH_AI_HOST",
    "AHNLICH_AI_PORT",
    "AHNLICH_AI_MODEL",
    "AHNLICH_MCP_READ_ONLY",
)


@pytest.fixture(autouse=True)
def clear_ahnlich_environment(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    for variable in AHNLICH_ENVIRONMENT_VARIABLES:
        monkeypatch.delenv(variable, raising=False)


def test_ai_is_the_default_profile() -> None:
    settings = Settings.from_env()

    assert settings.profile is Profile.AI
    assert settings.host == "127.0.0.1"
    assert settings.port == 1370
    assert settings.ai_model == DEFAULT_TEXT_MODEL


def test_db_profile_uses_db_defaults() -> None:
    settings = Settings.from_env(profile="db")

    assert settings.profile is Profile.DB
    assert settings.host == "127.0.0.1"
    assert settings.port == 1369
    assert settings.ai_model is None


def test_profile_reads_environment(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setenv("AHNLICH_PROFILE", "db")

    settings = Settings.from_env()

    assert settings.profile is Profile.DB


def test_explicit_profile_overrides_environment(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setenv("AHNLICH_PROFILE", "ai")

    settings = Settings.from_env(profile="db")

    assert settings.profile is Profile.DB


def test_profile_accepts_enum() -> None:
    settings = Settings.from_env(profile=Profile.DB)

    assert settings.profile is Profile.DB


def test_profile_is_case_insensitive() -> None:
    settings = Settings.from_env(profile=" DB ")

    assert settings.profile is Profile.DB


def test_ai_profile_reads_ai_environment(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setenv("AHNLICH_AI_HOST", "ai.internal")
    monkeypatch.setenv("AHNLICH_AI_PORT", "9000")
    monkeypatch.setenv(
        "AHNLICH_AI_MODEL",
        "all-minilm-l6-v2",
    )

    settings = Settings.from_env(profile="ai")

    assert settings.host == "ai.internal"
    assert settings.port == 9000
    assert settings.ai_model == "all-minilm-l6-v2"


def test_db_profile_reads_db_environment(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setenv("AHNLICH_DB_HOST", "db.internal")
    monkeypatch.setenv("AHNLICH_DB_PORT", "8001")

    settings = Settings.from_env(profile="db")

    assert settings.host == "db.internal"
    assert settings.port == 8001


def test_invalid_profile_is_rejected() -> None:
    with pytest.raises(
        ConfigurationError,
        match="Invalid Ahnlich profile",
    ):
        Settings.from_env(profile="automatic")


@pytest.mark.parametrize(
    ("profile", "variable"),
    [
        ("db", "AHNLICH_DB_PORT"),
        ("ai", "AHNLICH_AI_PORT"),
    ],
)
def test_non_integer_port_is_rejected(
    monkeypatch: pytest.MonkeyPatch,
    profile: str,
    variable: str,
) -> None:
    monkeypatch.setenv(variable, "not-a-port")

    with pytest.raises(
        ConfigurationError,
        match="must be an integer",
    ):
        Settings.from_env(profile=profile)


@pytest.mark.parametrize("port", [0, -1, 65536])
def test_out_of_range_port_is_rejected(
    monkeypatch: pytest.MonkeyPatch,
    port: int,
) -> None:
    monkeypatch.setenv("AHNLICH_DB_PORT", str(port))

    with pytest.raises(
        ConfigurationError,
        match="must be between 1 and 65535",
    ):
        Settings.from_env(profile="db")


def test_empty_host_is_rejected(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setenv("AHNLICH_DB_HOST", "")

    with pytest.raises(
        ConfigurationError,
        match="AHNLICH_DB_HOST cannot be empty",
    ):
        Settings.from_env(profile="db")


def test_ai_profile_requires_model(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setenv("AHNLICH_AI_MODEL", "")

    with pytest.raises(
        ConfigurationError,
        match="AHNLICH_AI_MODEL cannot be empty",
    ):
        Settings.from_env(profile="ai")


def test_db_profile_ignores_invalid_ai_configuration(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setenv("AHNLICH_AI_PORT", "invalid")
    monkeypatch.setenv("AHNLICH_AI_MODEL", "")

    settings = Settings.from_env(profile="db")

    assert settings.profile is Profile.DB
    assert settings.port == 1369


def test_ai_profile_ignores_invalid_db_configuration(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setenv("AHNLICH_DB_PORT", "invalid")

    settings = Settings.from_env(profile="ai")

    assert settings.profile is Profile.AI
    assert settings.port == 1370

@pytest.mark.parametrize(
    "model",
    tuple(TEXT_MODELS),
)
def test_ai_profile_accepts_supported_model(
    monkeypatch: pytest.MonkeyPatch,
    model: str,
) -> None:
    monkeypatch.setenv(
        "AHNLICH_AI_MODEL",
        model,
    )

    settings = Settings.from_env(profile="ai")

    assert settings.ai_model == model


def test_unsupported_ai_model_is_rejected(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setenv(
        "AHNLICH_AI_MODEL",
        "unknown-model",
    )

    with pytest.raises(
        ConfigurationError,
        match="Unsupported AI model",
    ):
        Settings.from_env(profile="ai")

@pytest.mark.parametrize(
    "value",
    ["1", "true", "TRUE", "yes", "on", " YES "],
)
def test_read_only_truthy_values(
    monkeypatch: pytest.MonkeyPatch,
    value: str,
) -> None:
    monkeypatch.setenv("AHNLICH_MCP_READ_ONLY", value)

    settings = Settings.from_env()

    assert settings.read_only is True


@pytest.mark.parametrize(
    "value",
    ["0", "false", "FALSE", "no", "off"],
)
def test_read_only_falsey_values(
    monkeypatch: pytest.MonkeyPatch,
    value: str,
) -> None:
    monkeypatch.setenv("AHNLICH_MCP_READ_ONLY", value)

    settings = Settings.from_env()

    assert settings.read_only is False


def test_read_only_defaults_to_false() -> None:
    settings = Settings.from_env()

    assert settings.read_only is False


def test_invalid_read_only_value(
    monkeypatch: pytest.MonkeyPatch,
) -> None:
    monkeypatch.setenv("AHNLICH_MCP_READ_ONLY", "sometimes")

    with pytest.raises(
        ConfigurationError,
        match="AHNLICH_MCP_READ_ONLY",
    ):
        Settings.from_env()