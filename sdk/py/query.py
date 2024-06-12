# pyre-strict
from dataclasses import dataclass
import typing
import serde_types as st
import bincode

class Algorithm:
    VARIANTS = []  # type: typing.Sequence[typing.Type[Algorithm]]

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, Algorithm)

    @staticmethod
    def bincode_deserialize(input: bytes) -> 'Algorithm':
        v, buffer = bincode.deserialize(input, Algorithm)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read");
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

Algorithm.VARIANTS = [
    Algorithm__EuclideanDistance,
    Algorithm__DotProductSimilarity,
    Algorithm__CosineSimilarity,
]


@dataclass(frozen=True)
class Array:
    v: st.uint8
    dim: typing.Tuple[st.uint64]
    data: typing.Sequence[st.float32]

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, Array)

    @staticmethod
    def bincode_deserialize(input: bytes) -> 'Array':
        v, buffer = bincode.deserialize(input, Array)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read");
        return v


@dataclass(frozen=True)
class Predicate:
    key: str
    value: str
    op: "PredicateOp"

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, Predicate)

    @staticmethod
    def bincode_deserialize(input: bytes) -> 'Predicate':
        v, buffer = bincode.deserialize(input, Predicate)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read");
        return v


class PredicateCondition:
    VARIANTS = []  # type: typing.Sequence[typing.Type[PredicateCondition]]

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, PredicateCondition)

    @staticmethod
    def bincode_deserialize(input: bytes) -> 'PredicateCondition':
        v, buffer = bincode.deserialize(input, PredicateCondition)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read");
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


class PredicateOp:
    VARIANTS = []  # type: typing.Sequence[typing.Type[PredicateOp]]

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, PredicateOp)

    @staticmethod
    def bincode_deserialize(input: bytes) -> 'PredicateOp':
        v, buffer = bincode.deserialize(input, PredicateOp)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read");
        return v


@dataclass(frozen=True)
class PredicateOp__Equals(PredicateOp):
    INDEX = 0  # type: int
    pass


@dataclass(frozen=True)
class PredicateOp__NotEquals(PredicateOp):
    INDEX = 1  # type: int
    pass

PredicateOp.VARIANTS = [
    PredicateOp__Equals,
    PredicateOp__NotEquals,
]


class Query:
    VARIANTS = []  # type: typing.Sequence[typing.Type[Query]]

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, Query)

    @staticmethod
    def bincode_deserialize(input: bytes) -> 'Query':
        v, buffer = bincode.deserialize(input, Query)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read");
        return v


@dataclass(frozen=True)
class Query__CreateStore(Query):
    INDEX = 0  # type: int
    store: str
    dimension: st.uint64
    create_predicates: typing.Sequence[str]
    error_if_exists: bool


@dataclass(frozen=True)
class Query__GetKey(Query):
    INDEX = 1  # type: int
    store: str
    keys: typing.Sequence["Array"]


@dataclass(frozen=True)
class Query__GetPred(Query):
    INDEX = 2  # type: int
    store: str
    condition: "PredicateCondition"


@dataclass(frozen=True)
class Query__GetSimN(Query):
    INDEX = 3  # type: int
    store: str
    search_input: "Array"
    closest_n: st.uint64
    algorithm: "Algorithm"
    condition: typing.Optional["PredicateCondition"]


@dataclass(frozen=True)
class Query__CreateIndex(Query):
    INDEX = 4  # type: int
    store: str
    predicates: typing.Sequence[str]


@dataclass(frozen=True)
class Query__DropIndex(Query):
    INDEX = 5  # type: int
    store: str
    predicates: typing.Sequence[str]
    error_if_not_exists: bool


@dataclass(frozen=True)
class Query__Set(Query):
    INDEX = 6  # type: int
    store: str
    inputs: typing.Sequence[typing.Tuple["Array", typing.Dict[str, str]]]


@dataclass(frozen=True)
class Query__DelKey(Query):
    INDEX = 7  # type: int
    store: str
    keys: typing.Sequence["Array"]


@dataclass(frozen=True)
class Query__DelPred(Query):
    INDEX = 8  # type: int
    store: str
    condition: "PredicateCondition"


@dataclass(frozen=True)
class Query__DropStore(Query):
    INDEX = 9  # type: int
    store: str
    error_if_not_exists: bool


@dataclass(frozen=True)
class Query__InfoServer(Query):
    INDEX = 10  # type: int
    pass


@dataclass(frozen=True)
class Query__ListStores(Query):
    INDEX = 11  # type: int
    pass


@dataclass(frozen=True)
class Query__ListClients(Query):
    INDEX = 12  # type: int
    pass


@dataclass(frozen=True)
class Query__Ping(Query):
    INDEX = 13  # type: int
    pass

Query.VARIANTS = [
    Query__CreateStore,
    Query__GetKey,
    Query__GetPred,
    Query__GetSimN,
    Query__CreateIndex,
    Query__DropIndex,
    Query__Set,
    Query__DelKey,
    Query__DelPred,
    Query__DropStore,
    Query__InfoServer,
    Query__ListStores,
    Query__ListClients,
    Query__Ping,
]


@dataclass(frozen=True)
class ServerQuery:
    queries: typing.Sequence["Query"]

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, ServerQuery)

    @staticmethod
    def bincode_deserialize(input: bytes) -> 'ServerQuery':
        v, buffer = bincode.deserialize(input, ServerQuery)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read");
        return v

