from __future__ import annotations

import json
from collections.abc import Awaitable, Callable
from typing import Any, NoReturn

from pydantic import BaseModel, ConfigDict, Field
from mcp.server.fastmcp.exceptions import ToolError
from mcp.types import ToolAnnotations

from ahnlich_mcp.backends import (
    AIBackend,
    AhnlichConnectionError,
    Backend,
    DBBackend,
    PredicateIndexNotFoundError,
    StoreNotFoundError,
)
from ahnlich_mcp.backends.base import AlgorithmName


ToolResponse = dict[str, Any] | list[dict[str, Any]]
Tool = Callable[..., Awaitable[ToolResponse]]


class TextEntry(BaseModel):
    model_config = ConfigDict(extra="forbid")

    content: str = Field(min_length=1)
    metadata: dict[str, str] = Field(default_factory=dict)

class EmbeddedEntry(BaseModel):
    model_config = ConfigDict(extra="forbid")

    embedding: list[float] = Field(min_length=1)
    metadata: dict[str, str] = Field(default_factory=dict)

def error_payload(
    error: Exception, *, backend: Backend,
    store_name: str | None = None,
) -> dict[str, Any]:
    if isinstance(error, AhnlichConnectionError):
        action = (
            "Start ahnlich-db and verify its configured address."
            if backend.profile_name == "db"
            else (
                "Start ahnlich-ai and the ahnlich-db instance configured "
                "for it."
            )
        )

        return {
            "status": "error",
            "error": str(error),
            "profile": backend.profile_name,
            "endpoint": f"{backend.host}:{backend.port}",
            "suggested_action": action,
        }

    if isinstance(error, StoreNotFoundError):
        name = store_name or error.store_name

        return {
            "status": "error",
            "error": f"Store {name!r} was not found.",
            "suggested_action": "Create it first with create_store.",
        }

    if isinstance(error, PredicateIndexNotFoundError):
        return {
            "status": "error",
            "error": str(error),
            "suggested_action": (
                "Create the metadata index first with "
                "create_predicate_index."
            ),
        }

    message = str(error).strip() or error.__class__.__name__

    return {
        "status": "error",
        "error": message,
    }

def raise_tool_error(
    error: Exception,
    *,
    backend: Backend,
    store_name: str | None = None,
) -> NoReturn:
    payload = error_payload(
        error,
        backend=backend,
        store_name=store_name,
    )

    raise ToolError(
        json.dumps(payload)
    )

def build_common_tools(backend: Backend) -> dict[str, Tool]:
    async def ping() -> dict[str, Any]:
        """Check whether the configured Ahnlich service is reachable."""
        try:
            available = await backend.ping()
        except Exception as error:
            raise_tool_error(
                error,
                backend=backend,
            )

        result: dict[str, Any] = {
            "status": "ok" if available else "error",
            "profile": backend.profile_name,
            "endpoint": f"{backend.host}:{backend.port}",
            "available": available,
        }

        if not available:
            result["suggested_action"] = (
                "Start the configured Ahnlich service and verify its address."
            )

        return result
    
    async def server_info() -> dict[str, Any]:
        """Return information about the configured Ahnlich service."""
        try:
            return await backend.server_info()
        except Exception as error:
            return raise_tool_error(error, backend=backend)

    async def list_stores() -> ToolResponse:
        """List stores available through the configured profile."""
        try:
            return await backend.list_stores()
        except Exception as error:
            return raise_tool_error(error, backend=backend)

    async def drop_store(
        store_name: str,
        error_if_not_exists: bool = True,
    ) -> dict[str, Any]:
        """Delete a store and all of its entries."""
        try:
            deleted = await backend.drop_store(
                store_name=store_name,
                error_if_not_exists=error_if_not_exists,
            )
        except Exception as error:
            return raise_tool_error(
                error,
                backend=backend,
                store_name=store_name,
            )

        return {
            "status": "dropped",
            "store_name": store_name,
            "deleted": deleted,
        }

    async def get_by_metadata(
        store_name: str,
        filter: dict[str, str],
    ) -> ToolResponse:
        """Retrieve entries matching indexed metadata."""
        try:
            return await backend.get_by_metadata(
                store_name=store_name,
                metadata_filter=filter,
            )
        except Exception as error:
            return raise_tool_error(
                error,
                backend=backend,
                store_name=store_name,
            )

    async def delete_by_metadata(
        store_name: str,
        filter: dict[str, str],
    ) -> dict[str, Any]:
        """Delete entries matching indexed metadata."""
        try:
            deleted = await backend.delete_by_metadata(
                store_name=store_name,
                metadata_filter=filter,
            )
        except Exception as error:
            return raise_tool_error(
                error,
                backend=backend,
                store_name=store_name,
            )

        return {
            "store_name": store_name,
            "deleted": deleted,
        }

    async def create_predicate_index(
        store_name: str,
        keys: list[str],
    ) -> dict[str, Any]:
        """Create indexes for metadata keys."""
        try:
            created = await backend.create_predicate_index(
                store_name=store_name,
                keys=keys,
            )
        except Exception as error:
            return raise_tool_error(
                error,
                backend=backend,
                store_name=store_name,
            )

        return {
            "status": "created",
            "store_name": store_name,
            "keys": keys,
            "created": created,
        }

    async def drop_predicate_index(
        store_name: str,
        keys: list[str],
    ) -> dict[str, Any]:
        """Remove indexes from metadata keys."""
        try:
            deleted = await backend.drop_predicate_index(
                store_name=store_name,
                keys=keys,
            )
        except Exception as error:
            return raise_tool_error(
                error,
                backend=backend,
                store_name=store_name,
            )

        return {
            "status": "dropped",
            "store_name": store_name,
            "keys": keys,
            "deleted": deleted,
        }

    return {
        "ping": ping,
        "server_info": server_info,
        "list_stores": list_stores,
        "drop_store": drop_store,
        "get_by_metadata": get_by_metadata,
        "delete_by_metadata": delete_by_metadata,
        "create_predicate_index": create_predicate_index,
        "drop_predicate_index": drop_predicate_index,
    }


def build_ai_tools(backend: AIBackend) -> dict[str, Tool]:
    async def create_store(
        store_name: str,
        predicate_keys: list[str] | None = None,
        error_if_exists: bool = True,
    ) -> dict[str, Any]:
        """Create a store that embeds raw text through ahnlich-ai."""
        keys = predicate_keys or []

        try:
            await backend.create_store(
                store_name=store_name,
                predicate_keys=keys,
                error_if_exists=error_if_exists,
            )
        except Exception as error:
            return raise_tool_error(
                error,
                backend=backend,
                store_name=store_name,
            )

        return {
            "status": "created",
            "store_name": store_name,
            "predicate_indexes": keys,
        }

    async def store_entries(
        store_name: str,
        entries: list[TextEntry],
    ) -> dict[str, Any]:
        """Embed and store raw text with optional metadata."""
        values = [entry.model_dump() for entry in entries]

        try:
            return await backend.store_entries(
                store_name=store_name,
                entries=values,
            )
        except Exception as error:
            return raise_tool_error(
                error,
                backend=backend,
                store_name=store_name,
            )

    async def similarity_search(
        store_name: str,
        query: str,
        top_k: int = 5,
        algorithm: AlgorithmName = "cosine",
        filter: dict[str, str] | None = None,
    ) -> ToolResponse:
        """Search by the meaning of a raw-text query."""
        try:
            return await backend.similarity_search(
                store_name=store_name,
                query=query,
                top_k=top_k,
                algorithm=algorithm,
                metadata_filter=filter,
            )
        except Exception as error:
            return raise_tool_error(
                error,
                backend=backend,
                store_name=store_name,
            )

    return {
        "create_store": create_store,
        "store_entries": store_entries,
        "similarity_search": similarity_search,
    }


def build_db_tools(backend: DBBackend) -> dict[str, Tool]:
    async def create_store(
        store_name: str,
        dimension: int,
        predicate_keys: list[str] | None = None,
        error_if_exists: bool = True,
    ) -> dict[str, Any]:
        """Create a store for user-provided embeddings."""
        keys = predicate_keys or []

        try:
            await backend.create_store(
                store_name=store_name,
                dimension=dimension,
                predicate_keys=keys,
                error_if_exists=error_if_exists,
            )
        except Exception as error:
            return raise_tool_error(
                error,
                backend=backend,
                store_name=store_name,
            )

        return {
            "status": "created",
            "store_name": store_name,
            "dimension": dimension,
            "predicate_indexes": keys,
        }

    async def store_entries(
        store_name: str,
        entries: list[EmbeddedEntry],
    ) -> dict[str, Any]:
        """Store user-provided embeddings with optional metadata."""
        values = [entry.model_dump() for entry in entries]

        try:
            return await backend.store_entries(
                store_name=store_name,
                entries=values,
            )
        except Exception as error:
            return raise_tool_error(
                error,
                backend=backend,
                store_name=store_name,
            )

    async def similarity_search(
        store_name: str,
        query_embedding: list[float],
        top_k: int = 5,
        algorithm: AlgorithmName = "cosine",
        filter: dict[str, str] | None = None,
    ) -> ToolResponse:
        """Search using a user-provided query embedding."""
        try:
            return await backend.similarity_search(
                store_name=store_name,
                query_embedding=query_embedding,
                top_k=top_k,
                algorithm=algorithm,
                metadata_filter=filter,
            )
        except Exception as error:
            return raise_tool_error(
                error,
                backend=backend,
                store_name=store_name,
            )

    return {
        "create_store": create_store,
        "store_entries": store_entries,
        "similarity_search": similarity_search,
    }


TOOL_ORDER = (
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
)

TOOL_ANNOTATIONS: dict[str, ToolAnnotations] = {
    "ping": ToolAnnotations(
        readOnlyHint=True,
        destructiveHint=False,
        idempotentHint=True,
        openWorldHint=False,
    ),
    "server_info": ToolAnnotations(
        readOnlyHint=True,
        destructiveHint=False,
        idempotentHint=True,
        openWorldHint=False,
    ),
    "create_store": ToolAnnotations(
        readOnlyHint=False,
        destructiveHint=False,
        idempotentHint=False,
        openWorldHint=False,
    ),
    "list_stores": ToolAnnotations(
        readOnlyHint=True,
        destructiveHint=False,
        idempotentHint=True,
        openWorldHint=False,
    ),
    "drop_store": ToolAnnotations(
        readOnlyHint=False,
        destructiveHint=True,
        idempotentHint=False,
        openWorldHint=False,
    ),
    "store_entries": ToolAnnotations(
        readOnlyHint=False,
        destructiveHint=False,
        idempotentHint=True,
        openWorldHint=False,
    ),
    "similarity_search": ToolAnnotations(
        readOnlyHint=True,
        destructiveHint=False,
        idempotentHint=True,
        openWorldHint=False,
    ),
    "get_by_metadata": ToolAnnotations(
        readOnlyHint=True,
        destructiveHint=False,
        idempotentHint=True,
        openWorldHint=False,
    ),
    "delete_by_metadata": ToolAnnotations(
        readOnlyHint=False,
        destructiveHint=True,
        idempotentHint=True,
        openWorldHint=False,
    ),
    "create_predicate_index": ToolAnnotations(
        readOnlyHint=False,
        destructiveHint=False,
        idempotentHint=True,
        openWorldHint=False,
    ),
    "drop_predicate_index": ToolAnnotations(
        readOnlyHint=False,
        destructiveHint=True,
        idempotentHint=False,
        openWorldHint=False,
    ),
}


def build_tools(backend: Backend) -> tuple[Tool, ...]:
    tools = build_common_tools(backend)

    if isinstance(backend, AIBackend):
        tools.update(build_ai_tools(backend))
    else:
        tools.update(build_db_tools(backend))

    return tuple(tools[name] for name in TOOL_ORDER)
