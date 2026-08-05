from __future__ import annotations

from typing import Any

from grpclib.client import Channel

from ahnlich_client_py.grpc.ai import query as ai_query
from ahnlich_client_py.grpc.ai.models import AiModel
from ahnlich_client_py.grpc.ai.preprocess import (
    PreprocessAction,
)
from ahnlich_client_py.grpc.keyval import (
    AiStoreEntry,
    StoreInput,
)
from ahnlich_client_py.grpc.services import ai_service

from ahnlich_mcp.backends.base import (
    AlgorithmName,
    AhnlichError,
    BaseBackend,
)


AI_MODELS = {
    "all-minilm-l6-v2": AiModel.ALL_MINI_LM_L6_V2,
}


AI_MODEL_NAMES = {
    model: name
    for name, model in AI_MODELS.items()
}


class AIBackend(BaseBackend):
    """ Perform raw-input operations through ahnlich-ai. """

    profile_name = "ai"

    def __init__(
        self, *,
        host: str,
        port: int,
        model: str,
    ) -> None:
        super().__init__(
            service_name="ahnlich-ai",
            host=host,
            port=port,
        )

        model_value = AI_MODELS.get(model)

        if model_value is None:
            supported = ", ".join(
                sorted(AI_MODELS)
            )

            raise ValueError(
                f"Unsupported AI model {model!r}. "
                f"Supported models: {supported}"
            )

        self.model_name = model
        self.model_value = model_value

    def _create_stub(
        self,
        channel: Channel,
    ) -> ai_service.AiServiceStub:
        """Create the generated Ahnlich AI gRPC client."""
        return ai_service.AiServiceStub(channel)

    async def ping(self) -> bool:
        """Return whether the AI proxy is reachable."""
        try:
            await self._call(
                "ping",
                ai_query.Ping(),
            )
        except AhnlichError:
            return False

        return True

    async def server_info(self) -> dict[str, Any]:
        """Return version and connection information from Ahnlich AI."""
        response = await self._call(
            "info_server",
            ai_query.InfoServer(),
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
            "model": self.model_name,
        }

    async def create_store(
        self,
        *,
        store_name: str,
        predicate_keys: list[str],
        error_if_exists: bool,
    ) -> None:
        """ Create an AI store using the configured embedding model. """
        self._validate_store_name(store_name)

        validated_predicates: list[str]

        if predicate_keys:
            validated_predicates = (
                self._validate_predicate_keys(
                    predicate_keys
                )
            )
        else:
            validated_predicates = []

        await self._call(
            "create_store",
            ai_query.CreateStore(
                store=store_name,
                query_model=self.model_value,
                index_model=self.model_value,
                predicates=validated_predicates,
                non_linear_indices=[],
                error_if_exists=error_if_exists,
                store_original=True,
            ),
            store_name=store_name,
        )

    async def list_stores(
        self,
    ) -> list[dict[str, Any]]:
        """List AI stores and their underlying DB information."""
        response = await self._call(
            "list_stores",
            ai_query.ListStores(),
        )

        stores: list[dict[str, Any]] = []

        for store in response.stores:
            entry_count: int | None = None
            size_in_bytes: int | None = None

            if store.db_info is not None:
                entry_count = store.db_info.len
                size_in_bytes = (
                    store.db_info.size_in_bytes
                )

            stores.append(
                {
                    "name": store.name,
                    "dimension": store.dimension,
                    "embedding_size": (
                        store.embedding_size
                    ),
                    "query_model": (
                        AI_MODEL_NAMES.get(
                            store.query_model,
                            getattr(
                                store.query_model,
                                "name",
                                str(store.query_model),
                            ),
                        )
                    ),
                    "index_model": (
                        AI_MODEL_NAMES.get(
                            store.index_model,
                            getattr(
                                store.index_model,
                                "name",
                                str(store.index_model),
                            ),
                        )
                    ),
                    "predicate_indexes": list(
                        store.predicate_indices
                    ),
                    "entry_count": entry_count,
                    "size_in_bytes": size_in_bytes,
                }
            )

        return stores

    async def drop_store(
        self,
        *,
        store_name: str,
        error_if_not_exists: bool,
    ) -> int:
        """Drop an AI store and return its deletion count."""
        self._validate_store_name(store_name)

        response = await self._call(
            "drop_store",
            ai_query.DropStore(
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
        """ Embed and store raw content. """
        self._validate_store_name(store_name)

        response = await self._call(
            "set",
            ai_query.Set(
                store=store_name,
                inputs=self._build_entries(entries),
                preprocess_action=(
                    PreprocessAction.NoPreprocessing
                ),
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
        query: str,
        top_k: int,
        algorithm: AlgorithmName,
        metadata_filter: dict[str, str] | None,
    ) -> list[dict[str, Any]]:
        """Embed a raw query and return semantically similar content."""
        self._validate_store_name(store_name)
        self._validate_top_k(top_k)

        if not isinstance(query, str):
            raise ValueError(
                "query must be a string"
            )

        if not query.strip():
            raise ValueError(
                "query must not be empty"
            )

        algorithm_value = self._resolve_algorithm(
            algorithm
        )

        condition = None

        if metadata_filter is not None:
            condition = self._build_condition(
                metadata_filter
            )

        response = await self._call(
            "get_sim_n",
            ai_query.GetSimN(
                store=store_name,
                search_input=StoreInput(
                    raw_string=query
                ),
                condition=condition,
                closest_n=top_k,
                algorithm=algorithm_value,
                preprocess_action=(
                    PreprocessAction.NoPreprocessing
                ),
                model_params={},
            ),
            store_name=store_name,
            predicate_operation=(
                metadata_filter is not None
            ),
        )

        results = [
            {
                "content": (
                    entry.key.raw_string
                    if entry.key is not None
                    else ""
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

        return sorted(
            results,
            key=lambda result: result["similarity"],
            reverse=True,
        )

    async def get_by_metadata(
        self,
        *,
        store_name: str,
        metadata_filter: dict[str, str],
    ) -> list[dict[str, Any]]:
        """Return raw entries matching an indexed metadata filter."""
        self._validate_store_name(store_name)

        response = await self._call(
            "get_pred",
            ai_query.GetPred(
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
                "content": entry.key.raw_string,
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
        """Delete entries matching an indexed metadata filter."""
        self._validate_store_name(store_name)

        response = await self._call(
            "del_pred",
            ai_query.DelPred(
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
        """Create metadata predicate indexes."""
        self._validate_store_name(store_name)
        validated_keys = (
            self._validate_predicate_keys(keys)
        )

        response = await self._call(
            "create_pred_index",
            ai_query.CreatePredIndex(
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
        """Drop metadata predicate indexes."""
        self._validate_store_name(store_name)
        validated_keys = (
            self._validate_predicate_keys(keys)
        )

        response = await self._call(
            "drop_pred_index",
            ai_query.DropPredIndex(
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

    def _build_entries(
        self,
        entries: list[dict[str, Any]],
    ) -> list[AiStoreEntry]:
        """Validate raw entries and convert them to protobuf messages."""
        if not isinstance(entries, list):
            raise ValueError(
                "entries must be a list"
            )

        if not entries:
            raise ValueError(
                "entries must contain at least one entry"
            )

        result: list[AiStoreEntry] = []

        for position, entry in enumerate(entries):
            if not isinstance(entry, dict):
                raise ValueError(
                    f"entries[{position}] must be a dictionary"
                )

            content = entry.get("content")
            metadata = entry.get("metadata", {})

            if (
                not isinstance(content, str)
                or not content.strip()
            ):
                raise ValueError(
                    f"entries[{position}].content must be "
                    "a non-empty string"
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
                AiStoreEntry(
                    key=StoreInput(
                        raw_string=content
                    ),
                    value=serialized_metadata,
                )
            )

        return result
