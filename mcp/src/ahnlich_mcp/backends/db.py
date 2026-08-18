from __future__ import annotations

from typing import Any
from math import isfinite

from ahnlich_client_py.grpc.db import query as db_query
from ahnlich_client_py.grpc.keyval import (
    DbStoreEntry,
    StoreKey,
)
from ahnlich_client_py.grpc.services import db_service

from ahnlich_mcp.backends.base import (
    AlgorithmName,
    BaseBackend,
)


class DBBackend(BaseBackend):
    """Store and search precomputed embeddings in ahnlich-db."""

    profile_name = "db"
    service_name = "ahnlich-db"
    query_module = db_query
    stub_type = db_service.DbServiceStub

    def _entry_identity(
        self,
        entry: Any,
        *,
        include_embeddings: bool = False,
    ) -> dict[str, Any]:
        embedding = entry.key.key

        if not include_embeddings:
            return {
                "dimension": len(embedding),
            }

        return {
            "embedding": list(embedding),
        }

    def _format_store(self, store: Any) -> dict[str, Any]:
        return {
            "name": store.name,
            "dimension": store.dimension,
            "predicate_indexes": list(store.predicate_indices),
            "entry_count": store.len,
            "size_in_bytes": store.size_in_bytes,
        }

    def _build_set_request(
        self, *,
        store_name: str,
        entries: list[dict[str, Any]],
    ) -> db_query.Set:
        return db_query.Set(
            store=store_name,
            inputs=self._build_entries(entries),
        )

    async def create_store(
        self, *,
        store_name: str,
        dimension: int,
        predicate_keys: list[str],
        error_if_exists: bool,
    ) -> None:
        self._validate_store_name(store_name)
        self._validate_dimension(dimension)

        validated_predicates = (
            self._validate_predicate_keys(
                predicate_keys,
                allow_empty=True,
            )
        )

        await self._call(
            "create_store",
            db_query.CreateStore(
                store=store_name,
                dimension=dimension,
                create_predicates=validated_predicates,
                non_linear_indices=[],
                error_if_exists=error_if_exists,
            ),
            store_name=store_name,
        )


    async def similarity_search(
        self,
        *,
        store_name: str,
        query_embedding: list[float],
        top_k: int,
        algorithm: AlgorithmName,
        metadata_filter: dict[str, str] | None,
        include_embeddings: bool = False,
    ) -> list[dict[str, Any]]:
        algorithm_value, condition = (
            self._prepare_similarity_search(
                store_name=store_name,
                top_k=top_k,
                algorithm=algorithm,
                metadata_filter=metadata_filter,
            )
        )

        embedding = self._validate_embedding(
            query_embedding,
            field_name="query_embedding",
        )

        request = db_query.GetSimN(
            store=store_name,
            search_input=StoreKey(key=embedding),
            condition=condition,
            closest_n=top_k,
            algorithm=algorithm_value,
        )

        return await self._execute_similarity_search(
            request=request,
            store_name=store_name,
            metadata_filter_applied=(
                metadata_filter is not None
            ),
            include_embeddings=include_embeddings,
        )

    @staticmethod
    def _validate_dimension(
        dimension: int,
    ) -> int:
        if (
            isinstance(dimension, bool)
            or not isinstance(dimension, int)
        ):
            raise ValueError(
                "dimension must be an integer"
            )

        if dimension < 1:
            raise ValueError(
                "dimension must be at least 1"
            )

        return dimension

    @staticmethod
    def _validate_embedding(
        embedding: list[float],
        *,
        field_name: str = "embedding",
    ) -> list[float]:
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

    def _build_entries(
        self,
        entries: list[dict[str, Any]],
    ) -> list[DbStoreEntry]:
        validated_entries = self._validate_entries(
            entries
        )
        result: list[DbStoreEntry] = []

        for position, entry in enumerate(
            validated_entries
        ):

            embedding = entry.get("embedding")
            metadata = entry.get("metadata", {})

            validated_embedding = (
                self._validate_embedding(
                    embedding,
                    field_name=(
                        f"entries[{position}].embedding"
                    ),
                )
            )
            serialized_metadata = (
                self._serialize_metadata(
                    metadata,
                    field_name=(
                        f"entries[{position}].metadata"
                    ),
                )
            )

            result.append(
                DbStoreEntry(
                    key=StoreKey(
                        key=validated_embedding
                    ),
                    value=serialized_metadata,
                )
            )

        return result