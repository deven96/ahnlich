from __future__ import annotations

from typing import Any

from grpclib.client import Channel

from ahnlich_client_py.grpc.db import query as db_query
from ahnlich_client_py.grpc.keyval import (
    DbStoreEntry,
    StoreKey,
)
from ahnlich_client_py.grpc.services import db_service

from ahnlich_mcp.backends.base import (
    AlgorithmName,
    AhnlichError,
    BaseBackend,
)


class DBBackend(BaseBackend):
    """Store and search precomputed embeddings in ahnlich-db."""

    profile_name = "db"

    def __init__(
        self, *,
        host: str,
        port: int,
    ) -> None:
        super().__init__(
            service_name="ahnlich-db",
            host=host,
            port=port,
        )

    def _create_stub(
        self,
        channel: Channel,
    ) -> db_service.DbServiceStub:
        return db_service.DbServiceStub(channel)

    async def ping(self) -> bool:
        try:
            await self._call(
                "ping",
                db_query.Ping(),
            )
        except AhnlichError:
            return False

        return True

    async def server_info(self) -> dict[str, Any]:
        response = await self._call(
            "info_server",
            db_query.InfoServer(),
        )
        info = response.info

        return {
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
                predicate_keys
            )
            if predicate_keys
            else []
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

    async def list_stores(
        self,
    ) -> list[dict[str, Any]]:
        response = await self._call(
            "list_stores",
            db_query.ListStores(),
        )

        return [
            {
                "name": store.name,
                "dimension": store.dimension,
                "predicate_indexes": list(
                    store.predicate_indices
                ),
                "entry_count": store.len,
                "size_in_bytes": (
                    store.size_in_bytes
                ),
            }
            for store in response.stores
        ]

    async def drop_store(
        self,
        *,
        store_name: str,
        error_if_not_exists: bool,
    ) -> int:
        self._validate_store_name(store_name)

        response = await self._call(
            "drop_store",
            db_query.DropStore(
                store=store_name,
                error_if_not_exists=(
                    error_if_not_exists
                ),
            ),
            store_name=store_name,
        )

        return response.deleted_count

    async def store_entries(
        self,
        *,
        store_name: str,
        entries: list[dict[str, Any]],
    ) -> dict[str, int]:
        self._validate_store_name(store_name)

        response = await self._call(
            "set",
            db_query.Set(
                store=store_name,
                inputs=self._build_entries(entries),
            ),
            store_name=store_name,
        )

        return {
            "inserted": response.upsert.inserted,
            "updated": response.upsert.updated,
        }

    async def similarity_search(
        self,
        *,
        store_name: str,
        query_embedding: list[float],
        top_k: int,
        algorithm: AlgorithmName,
        metadata_filter: dict[str, str] | None,
    ) -> list[dict[str, Any]]:
        self._validate_store_name(store_name)
        self._validate_top_k(top_k)

        embedding = self._validate_embedding(
            query_embedding,
            field_name="query_embedding",
        )
        algorithm_value = self._resolve_algorithm(
            algorithm
        )

        request = db_query.GetSimN(
            store=store_name,
            search_input=StoreKey(
                key=embedding
            ),
            closest_n=top_k,
            algorithm=algorithm_value,
        )

        if metadata_filter is not None:
            request.condition = self._build_condition(
                metadata_filter
            )

        response = await self._call(
            "get_sim_n",
            request,
            store_name=store_name,
            predicate_operation=(
                metadata_filter is not None
            ),
        )

        return [
            {
                "embedding": list(
                    entry.key.key
                ),
                "metadata": (
                    self._deserialize_metadata(
                        entry.value
                    )
                ),
                "similarity": float(
                    entry.similarity.value
                ),
            }
            for entry in response.entries
        ]

    async def get_by_metadata(
        self,
        *,
        store_name: str,
        metadata_filter: dict[str, str],
    ) -> list[dict[str, Any]]:
        self._validate_store_name(store_name)

        response = await self._call(
            "get_pred",
            db_query.GetPred(
                store=store_name,
                condition=self._build_condition(
                    metadata_filter
                ),
            ),
            store_name=store_name,
            predicate_operation=True,
        )

        return [
            {
                "embedding": list(
                    entry.key.key
                ),
                "metadata": (
                    self._deserialize_metadata(
                        entry.value
                    )
                ),
            }
            for entry in response.entries
        ]

    async def delete_by_metadata(
        self,
        *,
        store_name: str,
        metadata_filter: dict[str, str],
    ) -> int:
        self._validate_store_name(store_name)

        response = await self._call(
            "del_pred",
            db_query.DelPred(
                store=store_name,
                condition=self._build_condition(
                    metadata_filter
                ),
            ),
            store_name=store_name,
            predicate_operation=True,
        )

        return response.deleted_count

    async def create_predicate_index(
        self,
        *,
        store_name: str,
        keys: list[str],
    ) -> int:
        self._validate_store_name(store_name)
        validated_keys = (
            self._validate_predicate_keys(keys)
        )

        response = await self._call(
            "create_pred_index",
            db_query.CreatePredIndex(
                store=store_name,
                predicates=validated_keys,
            ),
            store_name=store_name,
        )

        return response.created_indexes

    async def drop_predicate_index(
        self,
        *,
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
            db_query.DropPredIndex(
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

    def _build_entries(
        self,
        entries: list[dict[str, Any]],
    ) -> list[DbStoreEntry]:
        if not isinstance(entries, list):
            raise ValueError(
                "entries must be a list"
            )

        if not entries:
            raise ValueError(
                "entries must contain at least one entry"
            )

        result: list[DbStoreEntry] = []

        for position, entry in enumerate(entries):
            if not isinstance(entry, dict):
                raise ValueError(
                    f"entries[{position}] must be a dictionary"
                )

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