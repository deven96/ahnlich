from __future__ import annotations

from typing import Any, Literal

from grpclib.client import Channel
from grpclib.const import Status
from grpclib.exceptions import GRPCError, StreamTerminatedError

from ahnlich_client_py.grpc.ai import query as ai_query
from ahnlich_client_py.grpc.ai.models import AiModel
from ahnlich_client_py.grpc.ai.preprocess import PreprocessAction
from ahnlich_client_py.grpc.algorithm.algorithms import Algorithm
from ahnlich_client_py.grpc.db import query as db_query
from ahnlich_client_py.grpc.keyval import (
    AiStoreEntry,
    StoreInput,
    StoreValue,
)
from ahnlich_client_py.grpc.metadata import MetadataValue
from ahnlich_client_py.grpc.predicates import (
    AndCondition,
    Equals,
    Predicate,
    PredicateCondition,
)
from ahnlich_client_py.grpc.services import ai_service, db_service

AlgorithmName = Literal["cosine", "euclidean", "dot_product"]

class AhnlichError(Exception):
    """Base exception for errors returned by Ahnlich."""

class AhnlichConnectionError(AhnlichError):
    """Raised when an Ahnlich service cannot be reached."""

class StoreNotFoundError(AhnlichError):
    """Raised when an operation references a missing store."""

    def __init__(self, store_name: str) -> None:
        self.store_name = store_name
        super().__init__(f"Store {store_name!r} was not found")

class PredicateIndexNotFoundError(AhnlichError):
    """Raised when a predicate operation uses an unindexed metadata key."""

MODEL_NAMES = {
    "all-minilm-l6-v2": AiModel.ALL_MINI_LM_L6_V2,
}

MODEL_ENUM_NAMES = {
    value: name for name, value in MODEL_NAMES.items()
}

ALGORITHMS = {
    "cosine": Algorithm.CosineSimilarity,
    "euclidean": Algorithm.EuclideanDistance,
    "dot_product": Algorithm.DotProductSimilarity,
}

CONNECTION_STATUSES = {
    Status.UNAVAILABLE,
    Status.DEADLINE_EXCEEDED,
}

class AhnlichClient:
    """Manage async connections to ahnlich-db and ahnlich-ai."""

    def __init__(self, db_host: str, db_port: int, ai_host: str, ai_port: int) -> None:
        self.db_host = db_host
        self.db_port = db_port
        self.ai_host = ai_host
        self.ai_port = ai_port

        self._db_channel: Channel | None = None
        self._ai_channel: Channel | None = None
        self._db_client: db_service.DbServiceStub | None = None
        self._ai_client: ai_service.AiServiceStub | None = None

    async def connect(self) -> None:
        """Create gRPC channels without requiring either server at startup."""
        if self._db_channel is None:
            self._db_channel = Channel(
                host=self.db_host,
                port=self.db_port,
            )
            self._db_client = db_service.DbServiceStub(self._db_channel)

        if self._ai_channel is None:
            self._ai_channel = Channel(
                host=self.ai_host,
                port=self.ai_port,
            )
            self._ai_client = ai_service.AiServiceStub(self._ai_channel)

    async def close(self) -> None:
        """Close both gRPC channels."""
        if self._db_channel is not None:
            self._db_channel.close()

        if self._ai_channel is not None:
            self._ai_channel.close()

        self._db_channel = None
        self._ai_channel = None
        self._db_client = None
        self._ai_client = None

    async def _call(
        self,
        service: Literal["db", "ai"],
        method_name: str,
        request: Any,
        *,
        store_name: str | None = None,
        predicate_operation: bool = False,
    ) -> Any:
        await self.connect()

        client = self._db_client if service == "db" else self._ai_client
        if client is None:
            raise AhnlichConnectionError(
                f"The Ahnlich {service.upper()} client is unavailable"
            )

        method = getattr(client, method_name)

        try:
            return await method(request)
        except GRPCError as error:
            translated = self._translate_grpc_error(
                error,
                store_name=store_name,
                predicate_operation=predicate_operation,
            )

            if isinstance(translated, AhnlichConnectionError):
                await self.close()

            raise translated from error
        except (
            OSError,
            ConnectionError,
            TimeoutError,
            StreamTerminatedError,
        ) as error:
            await self.close()
            raise AhnlichConnectionError(str(error)) from error

    @staticmethod
    def _translate_grpc_error(
        error: GRPCError,
        *,
        store_name: str | None,
        predicate_operation: bool,
    ) -> AhnlichError:
        message = error.message or str(error)
        lowered = message.lower()

        if error.status in CONNECTION_STATUSES:
            return AhnlichConnectionError(message)

        if (
            predicate_operation
            and "predicate" in lowered
            and (
                "index" in lowered
                or "not found" in lowered
                or "does not exist" in lowered
            )
        ):
            return PredicateIndexNotFoundError(message)

        if error.status == Status.NOT_FOUND and store_name is not None:
            return StoreNotFoundError(store_name)

        return AhnlichError(message)

    async def ping_db(self) -> bool:
        """Return whether the vector database is reachable."""
        try:
            await self._call("db", "ping", db_query.Ping())
        except AhnlichError:
            return False

        return True

    async def ping_ai(self) -> bool:
        """Return whether the AI proxy is reachable."""
        try:
            await self._call("ai", "ping", ai_query.Ping())
        except AhnlichError:
            return False

        return True

    async def server_info(self) -> dict[str, Any]:
        """Return information about the AI proxy."""
        response = await self._call(
            "ai",
            "info_server",
            ai_query.InfoServer(),
        )
        info = response.info

        return {
            "address": info.address,
            "version": info.version,
            "type": getattr(info.type, "name", str(info.type)).lower(),
            "limit": info.limit,
            "remaining": info.remaining,
        }

    async def create_store(
        self,
        store_name: str,
        model: str,
        predicate_keys: list[str],
        error_if_exists: bool,
    ) -> None:
        """Create an AI store that retains its original text inputs."""
        model_value = MODEL_NAMES.get(model)

        if model_value is None:
            supported = ", ".join(sorted(MODEL_NAMES))
            raise ValueError(
                f"Unsupported model {model!r}. Supported models: {supported}"
            )

        await self._call(
            "ai",
            "create_store",
            ai_query.CreateStore(
                store=store_name,
                query_model=model_value,
                index_model=model_value,
                predicates=predicate_keys,
                non_linear_indices=[],
                error_if_exists=error_if_exists,
                store_original=True,
            ),
            store_name=store_name,
        )

    async def list_stores(self) -> list[dict[str, Any]]:
        """List AI stores and convert their gRPC values to dictionaries."""
        response = await self._call(
            "ai",
            "list_stores",
            ai_query.ListStores(),
        )

        stores: list[dict[str, Any]] = []

        for store in response.stores:
            entry_count = None
            size_in_bytes = None

            if store.db_info is not None:
                entry_count = store.db_info.len
                size_in_bytes = store.db_info.size_in_bytes

            stores.append(
                {
                    "name": store.name,
                    "dimension": store.dimension,
                    "embedding_size": store.embedding_size,
                    "query_model": MODEL_ENUM_NAMES.get(
                        store.query_model,
                        getattr(
                            store.query_model,
                            "name",
                            str(store.query_model),
                        ),
                    ),
                    "index_model": MODEL_ENUM_NAMES.get(
                        store.index_model,
                        getattr(
                            store.index_model,
                            "name",
                            str(store.index_model),
                        ),
                    ),
                    "predicate_indexes": list(store.predicate_indices),
                    "entry_count": entry_count,
                    "size_in_bytes": size_in_bytes,
                }
            )

        return stores

    async def drop_store(
        self,
        store_name: str,
        error_if_not_exists: bool,
    ) -> int:
        """Drop an AI store and return the deletion count."""
        response = await self._call(
            "ai",
            "drop_store",
            ai_query.DropStore(
                store=store_name,
                error_if_not_exists=error_if_not_exists,
            ),
            store_name=store_name,
        )
        return response.deleted_count

    async def set(
        self,
        store_name: str,
        entries: list[dict[str, Any]],
    ) -> dict[str, int]:
        """Insert entries and return Ahnlich's inserted/updated counts."""
        response = await self._call(
            "ai",
            "set",
            ai_query.Set(
                store=store_name,
                inputs=self._build_entries(entries),
                preprocess_action=PreprocessAction.NoPreprocessing,
            ),
            store_name=store_name,
        )

        return {
            "inserted": response.upsert.inserted,
            "updated": response.upsert.updated,
        }

    async def upsert_content(
        self,
        store_name: str,
        entries: list[dict[str, Any]],
    ) -> dict[str, int]:
        """
        Insert or replace entries identified by their embedded content.

        Ahnlich's current Upsert RPC updates entries selected by metadata
        predicates. SET already performs key-based insert-or-update and returns
        both counts, which matches this MCP tool's content-oriented contract.
        """
        return await self.set(store_name, entries)

    async def similarity_search(
        self,
        store_name: str,
        query: str,
        top_k: int,
        algorithm: AlgorithmName,
        metadata_filter: dict[str, str] | None,
    ) -> list[dict[str, Any]]:
        """Perform semantic search through the AI proxy."""
        if top_k < 1:
            raise ValueError("top_k must be at least 1")

        algorithm_value = ALGORITHMS.get(algorithm)
        if algorithm_value is None:
            supported = ", ".join(sorted(ALGORITHMS))
            raise ValueError(
                f"Unsupported algorithm {algorithm!r}. "
                f"Supported algorithms: {supported}"
            )

        condition = None
        if metadata_filter is not None:
            condition = self._build_condition(metadata_filter)

        response = await self._call(
            "ai",
            "get_sim_n",
            ai_query.GetSimN(
                store=store_name,
                search_input=StoreInput(raw_string=query),
                condition=condition,
                closest_n=top_k,
                algorithm=algorithm_value,
                preprocess_action=PreprocessAction.NoPreprocessing,
                model_params={},
            ),
            store_name=store_name,
            predicate_operation=metadata_filter is not None,
        )

        results = [
            {
                "content": (
                    entry.key.raw_string
                    if entry.key is not None
                    else ""
                ),
                "metadata": self._deserialize_metadata(entry.value),
                "similarity": float(entry.similarity.value),
            }
            for entry in response.entries
        ]

        return sorted(
            results,
            key=lambda result: result["similarity"],
            reverse=True,
        )

    async def get_by_metadata(
        self,
        store_name: str,
        metadata_filter: dict[str, str],
    ) -> list[dict[str, Any]]:
        """Return entries matching an AND-combined metadata condition."""
        response = await self._call(
            "ai",
            "get_pred",
            ai_query.GetPred(
                store=store_name,
                condition=self._build_condition(metadata_filter),
            ),
            store_name=store_name,
            predicate_operation=True,
        )

        return [
            {
                "content": entry.key.raw_string,
                "metadata": self._deserialize_metadata(entry.value),
            }
            for entry in response.entries
        ]

    async def delete_by_metadata(
        self,
        store_name: str,
        metadata_filter: dict[str, str],
    ) -> int:
        """Delete entries matching an AND-combined metadata condition."""
        response = await self._call(
            "ai",
            "del_pred",
            ai_query.DelPred(
                store=store_name,
                condition=self._build_condition(metadata_filter),
            ),
            store_name=store_name,
            predicate_operation=True,
        )

        return response.deleted_count

    async def create_predicate_index(
        self,
        store_name: str,
        keys: list[str],
    ) -> int:
        """Create indexes for the supplied metadata keys."""
        if not keys:
            raise ValueError("keys must contain at least one metadata key")

        response = await self._call(
            "ai",
            "create_pred_index",
            ai_query.CreatePredIndex(
                store=store_name,
                predicates=keys,
            ),
            store_name=store_name,
        )

        return response.created_indexes

    async def drop_predicate_index(
        self,
        store_name: str,
        keys: list[str],
    ) -> int:
        """Drop indexes for the supplied metadata keys."""
        if not keys:
            raise ValueError("keys must contain at least one metadata key")

        response = await self._call(
            "ai",
            "drop_pred_index",
            ai_query.DropPredIndex(
                store=store_name,
                predicates=keys,
                error_if_not_exists=True,
            ),
            store_name=store_name,
            predicate_operation=True,
        )

        return response.deleted_count

    @staticmethod
    def _build_entries(
        entries: list[dict[str, Any]],
    ) -> list[AiStoreEntry]:
        if not entries:
            raise ValueError("entries must contain at least one entry")

        result: list[AiStoreEntry] = []

        for position, entry in enumerate(entries):
            content = entry.get("content")
            entry_metadata = entry.get("metadata", {})

            if not isinstance(content, str) or not content.strip():
                raise ValueError(
                    f"entries[{position}].content must be a non-empty string"
                )

            if not isinstance(entry_metadata, dict):
                raise ValueError(
                    f"entries[{position}].metadata must be a dictionary"
                )

            serialized_metadata: dict[str, MetadataValue] = {}

            for key, value in entry_metadata.items():
                if not isinstance(key, str) or not isinstance(value, str):
                    raise ValueError(
                        "Metadata keys and values must both be strings"
                    )

                serialized_metadata[key] = MetadataValue(
                    raw_string=value
                )

            result.append(
                AiStoreEntry(
                    key=StoreInput(raw_string=content),
                    value=StoreValue(value=serialized_metadata),
                )
            )

        return result

    @staticmethod
    def _deserialize_metadata(
        value: StoreValue,
    ) -> dict[str, str]:
        return {
            key: metadata_value.raw_string
            for key, metadata_value in value.value.items()
        }

    @staticmethod
    def _build_condition(
        metadata_filter: dict[str, str],
    ) -> PredicateCondition:
        if not metadata_filter:
            raise ValueError("filter must contain at least one condition")

        conditions: list[PredicateCondition] = []

        for key, value in metadata_filter.items():
            if not isinstance(key, str) or not isinstance(value, str):
                raise ValueError(
                    "Filter keys and values must both be strings"
                )

            conditions.append(
                PredicateCondition(
                    value=Predicate(
                        equals=Equals(
                            key=key,
                            value=MetadataValue(raw_string=value),
                        )
                    )
                )
            )

        condition = conditions[0]

        for next_condition in conditions[1:]:
            condition = PredicateCondition(
                and_=AndCondition(
                    left=condition,
                    right=next_condition,
                )
            )

        return condition