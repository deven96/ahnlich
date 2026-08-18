from __future__ import annotations

import argparse
import asyncio
import json
import logging
from collections.abc import Sequence, AsyncIterator
from contextlib import asynccontextmanager
from typing import Any

from mcp.server.fastmcp import FastMCP

from ahnlich_mcp.backends import Backend, create_backend
from ahnlich_mcp.config import (
    ConfigurationError,
    Profile,
    Settings,
)
from ahnlich_mcp.tools import build_tools, TOOL_ANNOTATIONS

logger = logging.getLogger(__name__)

AI_INSTRUCTIONS = (
    "Store and search raw text using ahnlich-ai. "
    "Create a store before adding entries. "
    "Use store_entries to embed and store text, and use "
    "similarity_search with a natural-language query. "
    "Metadata fields used in filters require predicate indexes."
)

DB_INSTRUCTIONS = (
    "Store and search user-provided embeddings using ahnlich-db. "
    "Create a store with the embedding dimension before adding entries. "
    "Use store_entries with embeddings, and use similarity_search with "
    "a query embedding. Metadata fields used in filters require "
    "predicate indexes."
)


def instructions_for(profile: Profile) -> str:
    if profile is Profile.DB:
        return DB_INSTRUCTIONS

    return AI_INSTRUCTIONS


def create_server(
    settings: Settings,
) -> tuple[FastMCP[Any], Backend]:
    backend = create_backend(settings)

    @asynccontextmanager
    async def lifespan(
        _: FastMCP[Any],
    ) -> AsyncIterator[None]:
        try:
            try:
                available = await backend.ping()
            except Exception as error:
                available = False
                logger.warning(
                    "Could not check %s profile at %s:%s: %s",
                    settings.profile.value,
                    settings.host,
                    settings.port,
                    error,
                )

            if not available:
                logger.warning(
                    "Ahnlich %s profile is unavailable at %s:%s. "
                    "The MCP server will remain running, but tool calls "
                    "will fail until the service is started.",
                    settings.profile.value,
                    settings.host,
                    settings.port,
                )

            yield
        finally:
            await backend.close()

    mcp = FastMCP(
        name="ahnlich-mcp",
        instructions=instructions_for(settings.profile),
        lifespan=lifespan,
    )

    for tool in build_tools(backend):
        mcp.tool(
            annotations=TOOL_ANNOTATIONS[tool.__name__]
        )(tool)

    return mcp, backend


async def run_doctor(settings: Settings) -> int:
    backend = create_backend(settings)

    result: dict[str, Any] = {
        "profile": settings.profile.value,
        "endpoint": f"{settings.host}:{settings.port}",
    }

    try:
        available = await backend.ping()

        if not available:
            result.update(
                {
                    "status": "error",
                    "available": False,
                    "message": (
                        "The configured Ahnlich service could not be reached."
                    ),
                    "suggested_action": startup_hint(
                        settings.profile
                    ),
                }
            )
            print(json.dumps(result, indent=2))
            return 1

        result.update(
            {
                "status": "ok",
                "available": True,
                "server_info": await backend.server_info(),
            }
        )
        print(json.dumps(result, indent=2))
        return 0

    except Exception as error:
        result.update(
            {
                "status": "error",
                "available": False,
                "message": str(error),
                "suggested_action": startup_hint(
                    settings.profile
                ),
            }
        )
        print(json.dumps(result, indent=2))
        return 1

    finally:
        await backend.close()


def startup_hint(profile: Profile) -> str:
    if profile is Profile.DB:
        return (
            "Start ahnlich-db and verify AHNLICH_DB_HOST "
            "and AHNLICH_DB_PORT."
        )

    return (
        "Start ahnlich-ai and its configured ahnlich-db instance, "
        "then verify AHNLICH_AI_HOST and AHNLICH_AI_PORT."
    )


def build_parser() -> argparse.ArgumentParser:
    parser = argparse.ArgumentParser(
        prog="ahnlich-mcp",
        description="Run or diagnose the Ahnlich MCP server.",
    )

    parser.add_argument(
        "command",
        nargs="?",
        choices=("serve", "doctor"),
        default="serve",
    )

    parser.add_argument(
        "--profile",
        choices=tuple(profile.value for profile in Profile),
        help=(
            "Ahnlich profile to use. Overrides AHNLICH_PROFILE. "
            "Defaults to ai."
        ),
    )

    return parser


def parse_args(
    argv: Sequence[str] | None = None,
) -> argparse.Namespace:
    return build_parser().parse_args(argv)


def main(argv: Sequence[str] | None = None) -> None:
    parser = build_parser()
    args = parser.parse_args(argv)

    try:
        settings = Settings.from_env(profile=args.profile)
    except ConfigurationError as error:
        parser.error(str(error))

    if args.command == "doctor":
        raise SystemExit(
            asyncio.run(run_doctor(settings))
        )

    mcp, _ = create_server(settings)
    mcp.run(transport="stdio")


if __name__ == "__main__":
    main()