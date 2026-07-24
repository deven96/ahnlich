from __future__ import annotations

import os
from typing import Any, Literal

from ahnlich_mcp.client import (
    AhnlichClient,
    AhnlichConnectionError,
    PredicateIndexNotFoundError,
    StoreNotFoundError,
)

ToolResponse = dict[str, Any] | list[dict[str, Any]]
SimilarityAlgorithm = Literal[
    "cosine",
    "euclidean",
    "dot_product",
]

client = AhnlichClient(
    db_host=os.getenv("AHNLICH_DB_HOST", "127.0.0.1"),
    db_port=int(os.getenv("AHNLICH_DB_PORT", "1369")),
    ai_host=os.getenv("AHNLICH_AI_HOST", "127.0.0.1"),
    ai_port=int(os.getenv("AHNLICH_AI_PORT", "1370")),
)

def error_response(
    error: Exception,
    *,
    store_name: str | None = None,
) -> dict[str, str]:
    """Convert an internal exception to an MCP-safe error dictionary."""
    if isinstance(error, AhnlichConnectionError):
        return {
            "error": (
                "Cannot connect to Ahnlich. Is it running? "
                "Start with: docker compose up -d"
            )
        }

    if isinstance(error, StoreNotFoundError):
        name = store_name or error.store_name
        return {
            "error": (
                f"Store {name!r} not found. "
                "Create it first with create_store."
            )
        }

    if isinstance(error, PredicateIndexNotFoundError):
        return {
            "error": (
                "A required predicate index was not found. "
                "Create it first with create_predicate_index."
            )
        }

    message = str(error).strip() or error.__class__.__name__
    return {"error": message}

async def ping() -> dict[str, Any]:
    """
    Check whether the Ahnlich database and AI proxy are reachable.

    Use this before other operations when diagnosing connection or startup
    problems.
    """
    db_available = await client.ping_db()
    ai_available = await client.ping_ai()

    result: dict[str, Any] = {
        "status": (
            "ok"
            if db_available and ai_available
            else "error"
        ),
        "db": db_available,
        "ai": ai_available,
    }

    errors: dict[str, str] = {}

    if not db_available:
        errors["db"] = (
            "Cannot reach ahnlich-db at the configured address."
        )

    if not ai_available:
        errors["ai"] = (
            "Cannot reach ahnlich-ai at the configured address."
        )

    if errors:
        result["errors"] = errors
        result["start_hint"] = "docker compose up -d"

    return result

async def server_info() -> dict[str, Any]:
    """
    Get version and configuration information from the Ahnlich AI proxy.

    Use this to inspect the running server and confirm its version and limits.
    """
    try:
        return await client.server_info()
    except Exception as error:
        return error_response(error)

async def create_store(
    store_name: str,
    model: str = "all-minilm-l6-v2",
    predicate_keys: list[str] | None = None,
    error_if_exists: bool = True,
) -> dict[str, Any]:
    """
    Create a vector store with automatic text embedding.

    Supply predicate_keys for metadata fields that will be used in filtered
    searches, metadata retrieval, or metadata-based deletion.
    """
    keys = predicate_keys or []

    try:
        await client.create_store(
            store_name=store_name,
            model=model,
            predicate_keys=keys,
            error_if_exists=error_if_exists,
        )
    except Exception as error:
        return error_response(error, store_name=store_name)

    return {
        "status": "created",
        "store_name": store_name,
        "model": model,
        "predicate_indexes": keys,
    }

async def list_stores() -> ToolResponse:
    """
    List all existing vector stores and their configurations.

    Use this to discover available stores before searching or managing data.
    """
    try:
        return await client.list_stores()
    except Exception as error:
        return error_response(error)

async def drop_store(
    store_name: str,
    error_if_not_exists: bool = True,
) -> dict[str, Any]:
    """
    Delete a vector store and all of its data.

    Use this only when the entire store and every entry in it should be
    permanently removed.
    """
    try:
        deleted = await client.drop_store(
            store_name=store_name,
            error_if_not_exists=error_if_not_exists,
        )
    except Exception as error:
        return error_response(error, store_name=store_name)

    return {
        "status": "dropped",
        "store_name": store_name,
        "deleted": deleted,
    }

async def store_content(
    store_name: str,
    entries: list[dict[str, Any]],
) -> dict[str, Any]:
    """
    Store text content with metadata.

    The text is automatically embedded by Ahnlich's AI proxy. Use this for
    indexing documents, notes, file descriptions, or any text that should be
    searched semantically later. Each entry must contain "content" and may
    contain a string-to-string "metadata" dictionary.
    """
    try:
        counts = await client.set(
            store_name=store_name,
            entries=entries,
        )
    except Exception as error:
        return error_response(error, store_name=store_name)

    return {"inserted": counts["inserted"]}

async def upsert_content(
    store_name: str,
    entries: list[dict[str, Any]],
) -> dict[str, Any]:
    """
    Insert or update text content with metadata.

    If identical content already exists, its metadata is updated. Use this
    when re-indexing content that may already be stored.
    """
    try:
        counts = await client.upsert_content(
            store_name=store_name,
            entries=entries,
        )
    except Exception as error:
        return error_response(error, store_name=store_name)

    return {
        "inserted": counts["inserted"],
        "updated": counts["updated"],
    }

async def similarity_search(
    store_name: str,
    query: str,
    top_k: int = 5,
    algorithm: SimilarityAlgorithm = "cosine",
    filter: dict[str, str] | None = None,
) -> ToolResponse:
    """
    Search for content semantically similar to a natural-language query.

    Optionally filter by indexed metadata fields. Use this to find documents,
    notes, or other stored text by meaning rather than exact keywords.
    """
    try:
        return await client.similarity_search(
            store_name=store_name,
            query=query,
            top_k=top_k,
            algorithm=algorithm,
            metadata_filter=filter,
        )
    except Exception as error:
        return error_response(error, store_name=store_name)

async def get_by_metadata(
    store_name: str,
    filter: dict[str, str],
) -> ToolResponse:
    """
    Retrieve entries matching metadata conditions without similarity search.

    All supplied metadata conditions are combined with AND. The corresponding
    metadata keys must have predicate indexes.
    """
    try:
        return await client.get_by_metadata(
            store_name=store_name,
            metadata_filter=filter,
        )
    except Exception as error:
        return error_response(error, store_name=store_name)

async def delete_by_metadata(
    store_name: str,
    filter: dict[str, str],
) -> dict[str, Any]:
    """
    Delete every entry matching the supplied metadata conditions.

    All conditions are combined with AND. Use this when a group of indexed
    entries should be removed without dropping the entire store.
    """
    try:
        deleted = await client.delete_by_metadata(
            store_name=store_name,
            metadata_filter=filter,
        )
    except Exception as error:
        return error_response(error, store_name=store_name)

    return {"deleted": deleted}

async def create_predicate_index(
    store_name: str,
    keys: list[str],
) -> dict[str, Any]:
    """
    Create predicate indexes for metadata keys.

    Use this before filtering, retrieving, or deleting entries by those
    metadata fields.
    """
    try:
        created = await client.create_predicate_index(
            store_name=store_name,
            keys=keys,
        )
    except Exception as error:
        return error_response(error, store_name=store_name)

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
    """
    Remove predicate indexes from metadata keys.

    This removes the indexes, not the stored entries or their metadata.
    """
    try:
        deleted = await client.drop_predicate_index(
            store_name=store_name,
            keys=keys,
        )
    except Exception as error:
        return error_response(error, store_name=store_name)

    return {
        "status": "dropped",
        "store_name": store_name,
        "keys": keys,
        "deleted": deleted,
    }

MCP_TOOLS = (
    ping,
    server_info,
    create_store,
    list_stores,
    drop_store,
    store_content,
    upsert_content,
    similarity_search,
    get_by_metadata,
    delete_by_metadata,
    create_predicate_index,
    drop_predicate_index,
)