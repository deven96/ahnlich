from __future__ import annotations

from unittest.mock import AsyncMock

import pytest

from ahnlich_mcp.config import Profile, Settings
from ahnlich_mcp.server import (
    AI_INSTRUCTIONS,
    DB_INSTRUCTIONS,
    create_server,
    instructions_for,
    parse_args,
    run_doctor,
)

EXPECTED_TOOLS = {
    "ping",
    "server_info",
    "create_store",
    "list_stores",
    "drop_store",
    "store_entries",
    "similarity_search",
    "get_by_metadata",
    "delete_by_metadata",
    "create_predicate_index",
    "drop_predicate_index",
}


def test_default_command_is_serve() -> None:
    args = parse_args([])

    assert args.command == "serve"
    assert args.profile is None


def test_profile_can_be_selected_from_cli() -> None:
    args = parse_args(["--profile", "db"])

    assert args.command == "serve"
    assert args.profile == "db"


def test_doctor_command_can_select_ai_profile() -> None:
    args = parse_args(["doctor", "--profile", "ai"])

    assert args.command == "doctor"
    assert args.profile == "ai"


def test_ai_profile_instructions() -> None:
    assert instructions_for(Profile.AI) == AI_INSTRUCTIONS
    assert "raw text" in AI_INSTRUCTIONS


def test_db_profile_instructions() -> None:
    assert instructions_for(Profile.DB) == DB_INSTRUCTIONS
    assert "user-provided embeddings" in DB_INSTRUCTIONS


@pytest.mark.asyncio
async def test_ai_server_registers_ai_tools() -> None:
    settings = Settings(
        profile=Profile.AI,
        host="127.0.0.1",
        port=1370,
        ai_model="all-minilm-l6-v2",
    )

    mcp, backend = create_server(settings)

    try:
        registered = await mcp.list_tools()
    finally:
        await backend.close()

    assert {
        tool.name for tool in registered
    } == EXPECTED_TOOLS


@pytest.mark.asyncio
async def test_db_server_registers_db_tools() -> None:
    settings = Settings(
        profile=Profile.DB,
        host="127.0.0.1",
        port=1369,
    )

    mcp, backend = create_server(settings)

    try:
        registered = await mcp.list_tools()
    finally:
        await backend.close()

    assert {
        tool.name for tool in registered
    } == EXPECTED_TOOLS


@pytest.mark.asyncio
async def test_doctor_returns_success(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    backend = AsyncMock()
    backend.ping.return_value = True
    backend.server_info.return_value = {
        "version": "test",
    }

    monkeypatch.setattr(
        "ahnlich_mcp.server.create_backend",
        lambda settings: backend,
    )

    settings = Settings(
        profile=Profile.DB,
        host="127.0.0.1",
        port=1369,
    )

    exit_code = await run_doctor(settings)
    output = capsys.readouterr().out

    assert exit_code == 0
    assert '"status": "ok"' in output
    backend.close.assert_awaited_once()


@pytest.mark.asyncio
async def test_doctor_returns_failure(
    monkeypatch: pytest.MonkeyPatch,
    capsys: pytest.CaptureFixture[str],
) -> None:
    backend = AsyncMock()
    backend.ping.return_value = False

    monkeypatch.setattr(
        "ahnlich_mcp.server.create_backend",
        lambda settings: backend,
    )

    settings = Settings(
        profile=Profile.AI,
        host="127.0.0.1",
        port=1370,
        ai_model="all-minilm-l6-v2",
    )

    exit_code = await run_doctor(settings)
    output = capsys.readouterr().out

    assert exit_code == 1
    assert '"status": "error"' in output
    backend.close.assert_awaited_once()