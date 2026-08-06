from __future__ import annotations

from typing import Any

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
    """Store and search raw text through ahnlich-ai."""

    profile_name = "ai"
    service_name = "ahnlich-ai"
    query_module = ai_query
    stub_type = ai_service.AiServiceStub

    def __init__(
        self, *,
        host: str,
        port: int,
        model: str,
    ) -> None:
        super().__init__(
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

    def _server_info_extras(self) -> dict[str, Any]:
        return {"model": self.model_name}

    def _entry_identity(self, entry: Any) -> dict[str, Any]:
        key = entry.key

        return {
            "content": (
                key.raw_string
                if key is not None
                else ""
            )
        }

    @staticmethod
    def _model_name(model: Any) -> str:
        return AI_MODEL_NAMES.get(
            model,
            getattr(model, "name", str(model)),
        )

    def _format_store(self, store: Any) -> dict[str, Any]:
        entry_count: int | None = None
        size_in_bytes: int | None = None

        if store.db_info is not None:
            entry_count = store.db_info.len
            size_in_bytes = store.db_info.size_in_bytes

        return {
            "name": store.name,
            "dimension": store.dimension,
            "embedding_size": store.embedding_size,
            "query_model": self._model_name(store.query_model),
            "index_model": self._model_name(store.index_model),
            "predicate_indexes": list(store.predicate_indices),
            "entry_count": entry_count,
            "size_in_bytes": size_in_bytes,
        }

    def _build_set_request(
        self, *,
        store_name: str,
        entries: list[dict[str, Any]],
    ) -> ai_query.Set:
        return ai_query.Set(
            store=store_name,
            inputs=self._build_entries(entries),
            preprocess_action=PreprocessAction.NoPreprocessing,
        )

    async def create_store(
        self, *,
        store_name: str,
        predicate_keys: list[str],
        error_if_exists: bool,
    ) -> None:
        self._validate_store_name(store_name)

        validated_predicates: list[str]

        validated_predicates = (
            self._validate_predicate_keys(
                predicate_keys,
                allow_empty=True,
            )
        )

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

    async def similarity_search(
        self, *,
        store_name: str,
        query: str,
        top_k: int,
        algorithm: AlgorithmName,
        metadata_filter: dict[str, str] | None,
    ) -> list[dict[str, Any]]:
        algorithm_value, condition = (
            self._prepare_similarity_search(
                store_name=store_name,
                top_k=top_k,
                algorithm=algorithm,
                metadata_filter=metadata_filter,
            )
        )

        if not isinstance(query, str):
            raise ValueError(
                "query must be a string"
            )

        if not query.strip():
            raise ValueError(
                "query must not be empty"
            )

        if metadata_filter is not None:
            condition = self._build_condition(
                metadata_filter
            )

        request = ai_query.GetSimN(
            store=store_name,
            search_input=StoreInput(raw_string=query),
            condition=condition,
            closest_n=top_k,
            algorithm=algorithm_value,
            preprocess_action=(
                PreprocessAction.NoPreprocessing
            ),
            model_params={},
        )

        return await self._execute_similarity_search(
            request=request,
            store_name=store_name,
            metadata_filter_applied=(
                metadata_filter is not None
            ),
            sort_descending=True,
        )

    def _build_entries(
        self,
        entries: list[dict[str, Any]],
    ) -> list[AiStoreEntry]:
        validated_entries = self._validate_entries(
            entries
        )
        result: list[AiStoreEntry] = []

        for position, entry in enumerate(validated_entries):
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
