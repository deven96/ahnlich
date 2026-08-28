from __future__ import annotations
from ahnlich_client_py.grpc.ai.models import AiModel

DEFAULT_TEXT_MODEL = "all-minilm-l6-v2"

TEXT_MODELS: dict[str, AiModel] = {
    "all-minilm-l6-v2": (
        AiModel.ALL_MINI_LM_L6_V2
    ),
    "all-minilm-l12-v2": (
        AiModel.ALL_MINI_LM_L12_V2
    ),
    "bge-base-en-v1.5": (
        AiModel.BGE_BASE_EN_V15
    ),
    "bge-large-en-v1.5": (
        AiModel.BGE_LARGE_EN_V15
    ),
    "jina-embeddings-v2-base-code": (
        AiModel.JINA_EMBEDDINGS_V2_BASE_CODE
    ),
}

TEXT_MODEL_NAMES: dict[AiModel, str] = {
    model: name
    for name, model in TEXT_MODELS.items()
}