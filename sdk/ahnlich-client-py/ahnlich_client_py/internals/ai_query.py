# pyre-strict
import typing
from dataclasses import dataclass

from ahnlich_client_py.internals import bincode
from ahnlich_client_py.internals import serde_types as st


class AIModel:
    VARIANTS = []  # type: typing.Sequence[typing.Type[AIModel]]

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, AIModel)

    @staticmethod
    def bincode_deserialize(input: bytes) -> "AIModel":
        v, buffer = bincode.deserialize(input, AIModel)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read")
        return v


@dataclass(frozen=True)
class AIModel__AllMiniLML6V2(AIModel):
    INDEX = 0  # type: int
    pass


@dataclass(frozen=True)
class AIModel__AllMiniLML12V2(AIModel):
    INDEX = 1  # type: int
    pass


@dataclass(frozen=True)
class AIModel__BGEBaseEnV15(AIModel):
    INDEX = 2  # type: int
    pass


@dataclass(frozen=True)
class AIModel__BGELargeEnV15(AIModel):
    INDEX = 3  # type: int
    pass


@dataclass(frozen=True)
class AIModel__Resnet50(AIModel):
    INDEX = 4  # type: int
    pass


@dataclass(frozen=True)
class AIModel__ClipVitB32Image(AIModel):
    INDEX = 5  # type: int
    pass


@dataclass(frozen=True)
class AIModel__ClipVitB32Text(AIModel):
    INDEX = 6  # type: int
    pass


AIModel.VARIANTS = [
    AIModel__AllMiniLML6V2,
    AIModel__AllMiniLML12V2,
    AIModel__BGEBaseEnV15,
    AIModel__BGELargeEnV15,
    AIModel__Resnet50,
    AIModel__ClipVitB32Image,
    AIModel__ClipVitB32Text,
]


class AIQuery:
    VARIANTS = []  # type: typing.Sequence[typing.Type[AIQuery]]

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, AIQuery)

    @staticmethod
    def bincode_deserialize(input: bytes) -> "AIQuery":
        v, buffer = bincode.deserialize(input, AIQuery)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read")
        return v


@dataclass(frozen=True)
class AIQuery__CreateStore(AIQuery):
    INDEX = 0  # type: int
    store: str
    query_model: "AIModel"
    index_model: "AIModel"
    predicates: typing.Sequence[str]
    non_linear_indices: typing.Sequence["NonLinearAlgorithm"]
    error_if_exists: bool
    store_original: bool


@dataclass(frozen=True)
class AIQuery__GetPred(AIQuery):
    INDEX = 1  # type: int
    store: str
    condition: "PredicateCondition"


@dataclass(frozen=True)
class AIQuery__GetSimN(AIQuery):
    INDEX = 2  # type: int
    store: str
    search_input: "StoreInput"
    condition: typing.Optional["PredicateCondition"]
    closest_n: st.uint64
    algorithm: "Algorithm"
    preprocess_action: "PreprocessAction"


@dataclass(frozen=True)
class AIQuery__CreatePredIndex(AIQuery):
    INDEX = 3  # type: int
    store: str
    predicates: typing.Sequence[str]


@dataclass(frozen=True)
class AIQuery__CreateNonLinearAlgorithmIndex(AIQuery):
    INDEX = 4  # type: int
    store: str
    non_linear_indices: typing.Sequence["NonLinearAlgorithm"]


@dataclass(frozen=True)
class AIQuery__DropPredIndex(AIQuery):
    INDEX = 5  # type: int
    store: str
    predicates: typing.Sequence[str]
    error_if_not_exists: bool


@dataclass(frozen=True)
class AIQuery__DropNonLinearAlgorithmIndex(AIQuery):
    INDEX = 6  # type: int
    store: str
    non_linear_indices: typing.Sequence["NonLinearAlgorithm"]
    error_if_not_exists: bool


@dataclass(frozen=True)
class AIQuery__Set(AIQuery):
    INDEX = 7  # type: int
    store: str
    inputs: typing.Sequence[
        typing.Tuple["StoreInput", typing.Dict[str, "MetadataValue"]]
    ]
    preprocess_action: "PreprocessAction"


@dataclass(frozen=True)
class AIQuery__DelKey(AIQuery):
    INDEX = 8  # type: int
    store: str
    key: "StoreInput"


@dataclass(frozen=True)
class AIQuery__DropStore(AIQuery):
    INDEX = 9  # type: int
    store: str
    error_if_not_exists: bool


@dataclass(frozen=True)
class AIQuery__GetKey(AIQuery):
    INDEX = 10  # type: int
    store: str
    keys: typing.Sequence["StoreInput"]


@dataclass(frozen=True)
class AIQuery__InfoServer(AIQuery):
    INDEX = 11  # type: int
    pass


@dataclass(frozen=True)
class AIQuery__ListClients(AIQuery):
    INDEX = 12  # type: int
    pass


@dataclass(frozen=True)
class AIQuery__ListStores(AIQuery):
    INDEX = 13  # type: int
    pass


@dataclass(frozen=True)
class AIQuery__PurgeStores(AIQuery):
    INDEX = 14  # type: int
    pass


@dataclass(frozen=True)
class AIQuery__Ping(AIQuery):
    INDEX = 15  # type: int
    pass


AIQuery.VARIANTS = [
    AIQuery__CreateStore,
    AIQuery__GetPred,
    AIQuery__GetSimN,
    AIQuery__CreatePredIndex,
    AIQuery__CreateNonLinearAlgorithmIndex,
    AIQuery__DropPredIndex,
    AIQuery__DropNonLinearAlgorithmIndex,
    AIQuery__Set,
    AIQuery__DelKey,
    AIQuery__DropStore,
    AIQuery__GetKey,
    AIQuery__InfoServer,
    AIQuery__ListClients,
    AIQuery__ListStores,
    AIQuery__PurgeStores,
    AIQuery__Ping,
]


@dataclass(frozen=True)
class AIServerQuery:
    queries: typing.Sequence["AIQuery"]
    trace_id: typing.Optional[str]

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, AIServerQuery)

    @staticmethod
    def bincode_deserialize(input: bytes) -> "AIServerQuery":
        v, buffer = bincode.deserialize(input, AIServerQuery)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read")
        return v


class AIStoreInputType:
    VARIANTS = []  # type: typing.Sequence[typing.Type[AIStoreInputType]]

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, AIStoreInputType)

    @staticmethod
    def bincode_deserialize(input: bytes) -> "AIStoreInputType":
        v, buffer = bincode.deserialize(input, AIStoreInputType)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read")
        return v


@dataclass(frozen=True)
class AIStoreInputType__RawString(AIStoreInputType):
    INDEX = 0  # type: int
    pass


@dataclass(frozen=True)
class AIStoreInputType__Image(AIStoreInputType):
    INDEX = 1  # type: int
    pass


AIStoreInputType.VARIANTS = [
    AIStoreInputType__RawString,
    AIStoreInputType__Image,
]


class Algorithm:
    VARIANTS = []  # type: typing.Sequence[typing.Type[Algorithm]]

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, Algorithm)

    @staticmethod
    def bincode_deserialize(input: bytes) -> "Algorithm":
        v, buffer = bincode.deserialize(input, Algorithm)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read")
        return v


@dataclass(frozen=True)
class Algorithm__EuclideanDistance(Algorithm):
    INDEX = 0  # type: int
    pass


@dataclass(frozen=True)
class Algorithm__DotProductSimilarity(Algorithm):
    INDEX = 1  # type: int
    pass


@dataclass(frozen=True)
class Algorithm__CosineSimilarity(Algorithm):
    INDEX = 2  # type: int
    pass


@dataclass(frozen=True)
class Algorithm__KDTree(Algorithm):
    INDEX = 3  # type: int
    pass


Algorithm.VARIANTS = [
    Algorithm__EuclideanDistance,
    Algorithm__DotProductSimilarity,
    Algorithm__CosineSimilarity,
    Algorithm__KDTree,
]


class MetadataValue:
    VARIANTS = []  # type: typing.Sequence[typing.Type[MetadataValue]]

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, MetadataValue)

    @staticmethod
    def bincode_deserialize(input: bytes) -> "MetadataValue":
        v, buffer = bincode.deserialize(input, MetadataValue)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read")
        return v


@dataclass(frozen=True)
class MetadataValue__RawString(MetadataValue):
    INDEX = 0  # type: int
    value: str


@dataclass(frozen=True)
class MetadataValue__Image(MetadataValue):
    INDEX = 1  # type: int
    value: typing.Sequence[st.uint8]


MetadataValue.VARIANTS = [
    MetadataValue__RawString,
    MetadataValue__Image,
]


class NonLinearAlgorithm:
    VARIANTS = []  # type: typing.Sequence[typing.Type[NonLinearAlgorithm]]

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, NonLinearAlgorithm)

    @staticmethod
    def bincode_deserialize(input: bytes) -> "NonLinearAlgorithm":
        v, buffer = bincode.deserialize(input, NonLinearAlgorithm)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read")
        return v


@dataclass(frozen=True)
class NonLinearAlgorithm__KDTree(NonLinearAlgorithm):
    INDEX = 0  # type: int
    pass


NonLinearAlgorithm.VARIANTS = [
    NonLinearAlgorithm__KDTree,
]


class Predicate:
    VARIANTS = []  # type: typing.Sequence[typing.Type[Predicate]]

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, Predicate)

    @staticmethod
    def bincode_deserialize(input: bytes) -> "Predicate":
        v, buffer = bincode.deserialize(input, Predicate)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read")
        return v


@dataclass(frozen=True)
class Predicate__Equals(Predicate):
    INDEX = 0  # type: int
    key: str
    value: "MetadataValue"


@dataclass(frozen=True)
class Predicate__NotEquals(Predicate):
    INDEX = 1  # type: int
    key: str
    value: "MetadataValue"


@dataclass(frozen=True)
class Predicate__In(Predicate):
    INDEX = 2  # type: int
    key: str
    value: typing.Sequence["MetadataValue"]


@dataclass(frozen=True)
class Predicate__NotIn(Predicate):
    INDEX = 3  # type: int
    key: str
    value: typing.Sequence["MetadataValue"]


Predicate.VARIANTS = [
    Predicate__Equals,
    Predicate__NotEquals,
    Predicate__In,
    Predicate__NotIn,
]


class PredicateCondition:
    VARIANTS = []  # type: typing.Sequence[typing.Type[PredicateCondition]]

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, PredicateCondition)

    @staticmethod
    def bincode_deserialize(input: bytes) -> "PredicateCondition":
        v, buffer = bincode.deserialize(input, PredicateCondition)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read")
        return v


@dataclass(frozen=True)
class PredicateCondition__Value(PredicateCondition):
    INDEX = 0  # type: int
    value: "Predicate"


@dataclass(frozen=True)
class PredicateCondition__And(PredicateCondition):
    INDEX = 1  # type: int
    value: typing.Tuple["PredicateCondition", "PredicateCondition"]


@dataclass(frozen=True)
class PredicateCondition__Or(PredicateCondition):
    INDEX = 2  # type: int
    value: typing.Tuple["PredicateCondition", "PredicateCondition"]


PredicateCondition.VARIANTS = [
    PredicateCondition__Value,
    PredicateCondition__And,
    PredicateCondition__Or,
]


class PreprocessAction:
    VARIANTS = []  # type: typing.Sequence[typing.Type[PreprocessAction]]

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, PreprocessAction)

    @staticmethod
    def bincode_deserialize(input: bytes) -> "PreprocessAction":
        v, buffer = bincode.deserialize(input, PreprocessAction)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read")
        return v


@dataclass(frozen=True)
class PreprocessAction__NoPreprocessing(PreprocessAction):
    INDEX = 0  # type: int
    pass


@dataclass(frozen=True)
class PreprocessAction__ModelPreprocessing(PreprocessAction):
    INDEX = 1  # type: int
    pass


PreprocessAction.VARIANTS = [
    PreprocessAction__NoPreprocessing,
    PreprocessAction__ModelPreprocessing,
]


class StoreInput:
    VARIANTS = []  # type: typing.Sequence[typing.Type[StoreInput]]

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, StoreInput)

    @staticmethod
    def bincode_deserialize(input: bytes) -> "StoreInput":
        v, buffer = bincode.deserialize(input, StoreInput)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read")
        return v


@dataclass(frozen=True)
class StoreInput__RawString(StoreInput):
    INDEX = 0  # type: int
    value: str


@dataclass(frozen=True)
class StoreInput__Image(StoreInput):
    INDEX = 1  # type: int
    value: typing.Sequence[st.uint8]


StoreInput.VARIANTS = [
    StoreInput__RawString,
    StoreInput__Image,
]
