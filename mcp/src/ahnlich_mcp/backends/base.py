from __future__ import annotations

from abc import ABC, abstractmethod
from typing import Any, ClassVar, Literal

from grpclib.client import Channel
from grpclib.const import Status
from grpclib.exceptions import (
    GRPCError,
    StreamTerminatedError,
)

from ahnlich_client_py.grpc.algorithm.algorithms import (
    Algorithm,
)
from ahnlich_client_py.grpc.keyval import StoreValue
from ahnlich_client_py.grpc.metadata import MetadataValue
from ahnlich_client_py.grpc.predicates import (
    AndCondition,
    Equals,
    Predicate,
    PredicateCondition,
)


AlgorithmName = Literal["cosine", "euclidean", "dot_product"]

ALGORITHMS: dict[AlgorithmName, Algorithm] = {
    "cosine": Algorithm.CosineSimilarity,
    "euclidean": Algorithm.EuclideanDistance,
    "dot_product": Algorithm.DotProductSimilarity,
}

MAX_TOP_K = 1024

CONNECTION_STATUSES = {
    Status.UNAVAILABLE,
    Status.DEADLINE_EXCEEDED,
}

class AhnlichError(Exception):
    """Base exception for errors returned by Ahnlich."""

class AhnlichConnectionError(AhnlichError):
    """Raised when an Ahnlich service cannot be reached."""

    def __init__(
        self, *,
        service_name: str,
        host: str,
        port: int,
        detail: str,
    ) -> None:
        self.service_name = service_name
        self.host = host
        self.port = port
        self.detail = detail

        super().__init__(
            f"Cannot connect to {service_name} at "
            f"{host}:{port}: {detail}"
        )

class StoreNotFoundError(AhnlichError):
    """Raised when an operation references a missing store."""

    def __init__(self, store_name: str) -> None:
        self.store_name = store_name

        super().__init__(
            f"Store {store_name!r} was not found"
        )

class PredicateIndexNotFoundError(AhnlichError):
    """Raised when a predicate operation uses an unindexed key."""

class BaseBackend(ABC):
    """Shared behavior for Ahnlich backends."""

    profile_name: ClassVar[str]
    service_name: ClassVar[str]
    query_module: ClassVar[Any]
    stub_type: ClassVar[type[Any]]

    def __init__(self, *, host: str, port: int) -> None:
        self.host = host
        self.port = port
        self._channel: Channel | None = None
        self._stub: Any | None = None

    @property
    def endpoint(self) -> str:
        return f"{self.host}:{self.port}"

    def _create_stub(self, channel: Channel) -> Any:
        return self.stub_type(channel)

    async def connect(self) -> None:
        if self._channel is not None:
            return

        self._channel = Channel(host=self.host, port=self.port)
        self._stub = self._create_stub(self._channel)

    async def close(self) -> None:
        if self._channel is not None:
            self._channel.close()

        self._channel = None
        self._stub = None

    async def ping(self) -> bool:
        try:
            await self._call(
                "ping",
                self.query_module.Ping(),
            )
        except AhnlichError:
            return False

        return True

    async def server_info(self) -> dict[str, Any]:
        response = await self._call(
            "info_server",
            self.query_module.InfoServer(),
        )
        info = response.info

        result = {
            "profile": self.profile_name,
            "service": self.service_name,
            "address": info.address,
            "version": info.version,
            "type": getattr(
                info.type,
                "name",
                str(info.type),
            ).lower(),
            "limit": info.limit,
            "remaining": info.remaining,
        }

        result.update(self._server_info_extras())

        return result

    def _server_info_extras(self) -> dict[str, Any]:
        return {}

    @abstractmethod
    def _entry_identity(
        self,
        entry: Any,
        *,
        include_embeddings: bool = False,
    ) -> dict[str, Any]:
        raise NotImplementedError

    @abstractmethod
    def _format_store(self, store: Any) -> dict[str, Any]:
        raise NotImplementedError

    @abstractmethod
    def _build_set_request(
        self,
        *,
        store_name: str,
        entries: list[dict[str, Any]],
    ) -> Any:
        raise NotImplementedError

    def _format_entry(
        self,
        entry: Any,
        *,
        include_embeddings: bool = False,
    ) -> dict[str, Any]:
        return {
            **self._entry_identity(
                entry,
                include_embeddings=include_embeddings,
            ),
            "metadata": self._deserialize_metadata(
                entry.value
            ),
        }

    def _format_search_entries(
        self,
        entries: list[Any],
        *,
        include_embeddings: bool = False,
    ) -> list[dict[str, Any]]:
        return [
            {
                **self._format_entry(
                    entry,
                    include_embeddings=include_embeddings,
                ),
                "similarity": float(
                    entry.similarity.value
                ),
            }
            for entry in entries
        ]

    async def _execute_similarity_search(
        self,
        *,
        request: Any,
        store_name: str,
        metadata_filter_applied: bool,
        include_embeddings: bool = False,
    ) -> list[dict[str, Any]]:
        response = await self._call(
            "get_sim_n",
            request,
            store_name=store_name,
            predicate_operation=metadata_filter_applied,
        )

        return self._format_search_entries(
            response.entries,
            include_embeddings=include_embeddings,
        )

    async def list_stores(self) -> list[dict[str, Any]]:
        response = await self._call(
            "list_stores",
            self.query_module.ListStores(),
        )

        return [
            self._format_store(store)
            for store in response.stores
        ]

    async def store_entries(
        self, *,
        store_name: str,
        entries: list[dict[str, Any]],
    ) -> dict[str, int]:
        self._validate_store_name(store_name)

        request = self._build_set_request(store_name=store_name, entries=entries)

        return await self._execute_store_request(store_name=store_name, request=request)


    async def _execute_store_request(
        self, *,
        store_name: str,
        request: Any,
    ) -> dict[str, int]:
        response = await self._call("set", request, store_name=store_name)

        return {
            "inserted": response.upsert.inserted,
            "updated": response.upsert.updated,
        }

    async def drop_store(self, *, store_name: str, error_if_not_exists: bool) -> int:
        self._validate_store_name(store_name)

        response = await self._call(
            "drop_store",
            self.query_module.DropStore(
                store=store_name,
                error_if_not_exists=(
                    error_if_not_exists
                ),
            ),
            store_name=store_name,
        )

        return response.deleted_count

    async def get_by_metadata(
        self,
        *,
        store_name: str,
        metadata_filter: dict[str, str],
        include_embeddings: bool = False,
    ) -> list[dict[str, Any]]:
        self._validate_store_name(store_name)

        response = await self._call(
            "get_pred",
            self.query_module.GetPred(
                store=store_name,
                condition=self._build_condition(
                    metadata_filter
                ),
            ),
            store_name=store_name,
            predicate_operation=True,
        )

        return [
            self._format_entry(
                entry,
                include_embeddings=include_embeddings,
            )
            for entry in response.entries
        ]

    async def delete_by_metadata(
        self, *,
        store_name: str,
        metadata_filter: dict[str, str],
    ) -> int:
        self._validate_store_name(store_name)

        response = await self._call(
            "del_pred",
            self.query_module.DelPred(
                store=store_name,
                condition=self._build_condition(
                    metadata_filter
                ),
            ),
            store_name=store_name,
            predicate_operation=True,
        )

        return response.deleted_count

    async def create_predicate_index(self, *, store_name: str, keys: list[str]) -> int:
        self._validate_store_name(store_name)
        validated_keys = (
            self._validate_predicate_keys(keys)
        )

        response = await self._call(
            "create_pred_index",
            self.query_module.CreatePredIndex(
                store=store_name,
                predicates=validated_keys,
            ),
            store_name=store_name,
            predicate_operation=True,
        )

        return response.created_indexes

    async def drop_predicate_index(
        self, *,
        store_name: str,
        keys: list[str],
        error_if_not_exists: bool = True,
    ) -> int:
        self._validate_store_name(store_name)
        validated_keys = (
            self._validate_predicate_keys(keys)
        )

        response = await self._call(
            "drop_pred_index",
            self.query_module.DropPredIndex(
                store=store_name,
                predicates=validated_keys,
                error_if_not_exists=(
                    error_if_not_exists
                ),
            ),
            store_name=store_name,
            predicate_operation=True,
        )

        return response.deleted_count
    
    async def _call(
        self,
        method_name: str,
        request: Any,
        *,
        store_name: str | None = None,
        predicate_operation: bool = False,
    ) -> Any:
        """Call a gRPC method and translate transport errors."""
        await self.connect()

        if self._stub is None:
            raise AhnlichConnectionError(
                service_name=self.service_name,
                host=self.host,
                port=self.port,
                detail="The gRPC client was not created",
            )

        method = getattr(self._stub, method_name)

        try:
            return await method(request)
        except GRPCError as error:
            translated = self._translate_grpc_error(
                error,
                store_name=store_name,
                predicate_operation=predicate_operation,
            )

            if isinstance(
                translated,
                AhnlichConnectionError,
            ):
                await self.close()

            raise translated from error
        except (
            OSError,
            ConnectionError,
            TimeoutError,
            StreamTerminatedError,
        ) as error:
            await self.close()

            raise AhnlichConnectionError(
                service_name=self.service_name,
                host=self.host,
                port=self.port,
                detail=str(error),
            ) from error

    def _translate_grpc_error(
        self,
        error: GRPCError,
        *,
        store_name: str | None,
        predicate_operation: bool,
    ) -> AhnlichError:
        """Translate a raw gRPC error into a domain-specific exception."""
        message = error.message or str(error)
        lowered_message = message.lower()

        if error.status in CONNECTION_STATUSES:
            return AhnlichConnectionError(
                service_name=self.service_name,
                host=self.host,
                port=self.port,
                detail=message,
            )

        if (
            predicate_operation
            and "predicate" in lowered_message
            and (
                "index" in lowered_message
                or "not found" in lowered_message
                or "does not exist" in lowered_message
            )
        ):
            return PredicateIndexNotFoundError(message)

        if (
            error.status == Status.NOT_FOUND
            and store_name is not None
        ):
            return StoreNotFoundError(store_name)

        return AhnlichError(message)

    @staticmethod
    def _validate_store_name(store_name: str) -> str:
        if not isinstance(store_name, str):
            raise ValueError(
                "store_name must be a string"
            )

        if not store_name.strip():
            raise ValueError(
                "store_name must not be empty"
            )

        return store_name

    @staticmethod
    def _validate_top_k(top_k: int) -> int:
        if isinstance(top_k, bool) or not isinstance(top_k, int):
            raise ValueError(
                "top_k must be an integer"
            )

        if top_k < 1:
            raise ValueError(
                "top_k must be at least 1"
            )

        if top_k > MAX_TOP_K:
            raise ValueError(f"top_k must not exceed {MAX_TOP_K}")

        return top_k

    @staticmethod
    def _resolve_algorithm(
        algorithm: str,
    ) -> Algorithm:
        algorithm_value = ALGORITHMS.get(algorithm)

        if algorithm_value is None:
            supported = ", ".join(sorted(ALGORITHMS))

            raise ValueError(
                f"Unsupported algorithm {algorithm!r}. "
                f"Supported algorithms: {supported}"
            )

        return algorithm_value

    @staticmethod
    def _validate_entries(entries: list[dict[str, Any]]) -> list[dict[str, Any]]:
        if not isinstance(entries, list):
            raise ValueError(
                "entries must be a list"
            )

        if not entries:
            raise ValueError(
                "entries must contain at least one entry"
            )

        for position, entry in enumerate(entries):
            if not isinstance(entry, dict):
                raise ValueError(
                    f"entries[{position}] must be a dictionary"
                )

        return entries

    @staticmethod
    def _validate_predicate_keys(
        keys: list[str],
        *,
        allow_empty: bool = False,
    ) -> list[str]:
        if not isinstance(keys, list):
            raise ValueError(
                "keys must be a list of strings"
            )

        if not keys:
            if allow_empty:
                return []

            raise ValueError(
                "keys must contain at least one metadata key"
            )

        validated: list[str] = []
        seen: set[str] = set()

        for position, key in enumerate(keys):
            if not isinstance(key, str) or not key.strip():
                raise ValueError(
                    f"keys[{position}] must be a non-empty string"
                )

            if key in seen:
                raise ValueError(
                    f"Duplicate metadata key {key!r}"
                )

            seen.add(key)
            validated.append(key)

        return validated
    
    @staticmethod
    def _serialize_metadata(
        metadata: dict[str, str],
        *,
        field_name: str = "metadata",
    ) -> StoreValue:
        if not isinstance(metadata, dict):
            raise ValueError(
                f"{field_name} must be a dictionary"
            )

        serialized: dict[str, MetadataValue] = {}

        for key, value in metadata.items():
            if not isinstance(key, str) or not key.strip():
                raise ValueError(
                    f"{field_name} keys must be non-empty strings"
                )

            if not isinstance(value, str):
                raise ValueError(
                    f"{field_name}[{key!r}] must be a string"
                )

            serialized[key] = MetadataValue(
                raw_string=value
            )

        return StoreValue(value=serialized)

    def _prepare_similarity_search(
        self,
        *,
        store_name: str,
        top_k: int,
        algorithm: AlgorithmName,
        metadata_filter: dict[str, str] | None,
    ) -> tuple[Algorithm, PredicateCondition | None]:
        self._validate_store_name(store_name)
        self._validate_top_k(top_k)

        algorithm_value = self._resolve_algorithm(
            algorithm
        )
        condition = (
            self._build_condition(metadata_filter)
            if metadata_filter is not None
            else None
        )

        return algorithm_value, condition

    @staticmethod
    def _deserialize_metadata(
        value: StoreValue,
    ) -> dict[str, str]:
        return {
            key: metadata_value.raw_string
            for key, metadata_value in value.value.items()
        }

    @classmethod
    def _build_condition(
        cls,
        metadata_filter: dict[str, str],
    ) -> PredicateCondition:
        """Build an AND-combined metadata predicate."""
        if not isinstance(metadata_filter, dict):
            raise ValueError(
                "metadata_filter must be a dictionary"
            )

        if not metadata_filter:
            raise ValueError(
                "metadata_filter must contain at least one condition"
            )

        conditions: list[PredicateCondition] = []

        for key, value in metadata_filter.items():
            if not isinstance(key, str) or not key.strip():
                raise ValueError(
                    "Metadata filter keys must be non-empty strings"
                )

            if not isinstance(value, str):
                raise ValueError(
                    f"Metadata filter value for {key!r} must be a string"
                )

            conditions.append(
                PredicateCondition(
                    value=Predicate(
                        equals=Equals(
                            key=key,
                            value=MetadataValue(
                                raw_string=value
                            ),
                        )
                    )
                )
            )

        combined = conditions[0]

        for next_condition in conditions[1:]:
            combined = PredicateCondition(
                and_=AndCondition(
                    left=combined,
                    right=next_condition,
                )
            )

        return combined
