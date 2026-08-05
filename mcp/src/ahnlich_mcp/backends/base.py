from __future__ import annotations

from abc import ABC, abstractmethod
from math import isfinite
from typing import Any, Literal

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
    """
    Shared gRPC and validation behavior for Ahnlich backends.
    DBBackend will create a DB stub, while AIBackend will create an AI stub.
    """

    def __init__(
        self, *,
        service_name: str,
        host: str,
        port: int,
    ) -> None:
        self.service_name = service_name
        self.host = host
        self.port = port

        self._channel: Channel | None = None
        self._stub: Any | None = None

    @property
    def endpoint(self) -> str:
        """Return the configured endpoint in host:port form."""
        return f"{self.host}:{self.port}"

    @abstractmethod
    def _create_stub(self, channel: Channel) -> Any:
        """
        Create the generated gRPC service stub.
        """

    async def connect(self) -> None:
        """
        Create the channel and generated client stub on demand.s
        """
        if self._channel is not None:
            return

        self._channel = Channel(
            host=self.host,
            port=self.port,
        )
        self._stub = self._create_stub(self._channel)

    async def close(self) -> None:
        """Close the gRPC channel and clear the generated client."""
        if self._channel is not None:
            self._channel.close()

        self._channel = None
        self._stub = None

    async def _call(
        self,
        method_name: str,
        request: Any,
        *,
        store_name: str | None = None,
        predicate_operation: bool = False,
    ) -> Any:
        """
        Call a generated gRPC method and translate transport errors.

        `store_name` allows a NOT_FOUND response to become a more useful
        StoreNotFoundError.

        `predicate_operation` allows errors about missing metadata indexes to
        become PredicateIndexNotFoundError.
        """
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
        """Ensure that a store name is a non-empty string."""
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
        """Ensure that a similarity-search result count is valid."""
        if isinstance(top_k, bool) or not isinstance(top_k, int):
            raise ValueError(
                "top_k must be an integer"
            )

        if top_k < 1:
            raise ValueError(
                "top_k must be at least 1"
            )

        return top_k

    @staticmethod
    def _resolve_algorithm(
        algorithm: str,
    ) -> Algorithm:
        """Convert a public algorithm name into its protobuf enum."""
        algorithm_value = ALGORITHMS.get(algorithm)

        if algorithm_value is None:
            supported = ", ".join(sorted(ALGORITHMS))

            raise ValueError(
                f"Unsupported algorithm {algorithm!r}. "
                f"Supported algorithms: {supported}"
            )

        return algorithm_value

    @staticmethod
    def _validate_embedding(
        embedding: list[float],
        *,
        field_name: str = "embedding",
    ) -> list[float]:
        """
        Validate an embedding and normalize numeric values to floats.s
        """
        if not isinstance(embedding, list):
            raise ValueError(
                f"{field_name} must be a list of numbers"
            )

        if not embedding:
            raise ValueError(
                f"{field_name} must not be empty"
            )

        normalized: list[float] = []

        for position, value in enumerate(embedding):
            if (
                isinstance(value, bool)
                or not isinstance(value, (int, float))
            ):
                raise ValueError(
                    f"{field_name}[{position}] must be a number"
                )

            numeric_value = float(value)

            if not isfinite(numeric_value):
                raise ValueError(
                    f"{field_name}[{position}] must be finite"
                )

            normalized.append(numeric_value)

        return normalized

    @staticmethod
    def _validate_predicate_keys(
        keys: list[str],
    ) -> list[str]:
        """Validate metadata keys used to create or remove indexes."""
        if not isinstance(keys, list):
            raise ValueError(
                "keys must be a list of strings"
            )

        if not keys:
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
        """Convert a string dictionary into Ahnlich metadata values."""
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

    @staticmethod
    def _deserialize_metadata(
        value: StoreValue,
    ) -> dict[str, str]:
        """Convert Ahnlich metadata into a plain string dictionary."""
        return {
            key: metadata_value.raw_string
            for key, metadata_value in value.value.items()
        }

    @classmethod
    def _build_condition(
        cls,
        metadata_filter: dict[str, str],
    ) -> PredicateCondition:
        """
        Build an AND-combined Ahnlich metadata predicate.

        A filter such as:

            {"type": "pdf", "directory": "Downloads"}

        becomes:

            type == "pdf" AND directory == "Downloads"
        """
        if not isinstance(metadata_filter, dict):
            raise ValueError(
                "filter must be a dictionary"
            )

        if not metadata_filter:
            raise ValueError(
                "filter must contain at least one condition"
            )

        conditions: list[PredicateCondition] = []

        for key, value in metadata_filter.items():
            if not isinstance(key, str) or not key.strip():
                raise ValueError(
                    "Filter keys must be non-empty strings"
                )

            if not isinstance(value, str):
                raise ValueError(
                    f"Filter value for {key!r} must be a string"
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
