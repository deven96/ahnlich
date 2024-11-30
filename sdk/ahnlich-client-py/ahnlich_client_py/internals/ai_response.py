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


class AIServerResponse:
    VARIANTS = []  # type: typing.Sequence[typing.Type[AIServerResponse]]

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, AIServerResponse)

    @staticmethod
    def bincode_deserialize(input: bytes) -> "AIServerResponse":
        v, buffer = bincode.deserialize(input, AIServerResponse)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read")
        return v


@dataclass(frozen=True)
class AIServerResponse__Unit(AIServerResponse):
    INDEX = 0  # type: int
    pass


@dataclass(frozen=True)
class AIServerResponse__Pong(AIServerResponse):
    INDEX = 1  # type: int
    pass


@dataclass(frozen=True)
class AIServerResponse__ClientList(AIServerResponse):
    INDEX = 2  # type: int
    value: typing.Sequence["ConnectedClient"]


@dataclass(frozen=True)
class AIServerResponse__StoreList(AIServerResponse):
    INDEX = 3  # type: int
    value: typing.Sequence["AIStoreInfo"]


@dataclass(frozen=True)
class AIServerResponse__InfoServer(AIServerResponse):
    INDEX = 4  # type: int
    value: "ServerInfo"


@dataclass(frozen=True)
class AIServerResponse__Set(AIServerResponse):
    INDEX = 5  # type: int
    value: "StoreUpsert"


@dataclass(frozen=True)
class AIServerResponse__Get(AIServerResponse):
    INDEX = 6  # type: int
    value: typing.Sequence[
        typing.Tuple[typing.Optional["StoreInput"], typing.Dict[str, "MetadataValue"]]
    ]


@dataclass(frozen=True)
class AIServerResponse__GetSimN(AIServerResponse):
    INDEX = 7  # type: int
    value: typing.Sequence[
        typing.Tuple[
            typing.Optional["StoreInput"],
            typing.Dict[str, "MetadataValue"],
            "Similarity",
        ]
    ]


@dataclass(frozen=True)
class AIServerResponse__Del(AIServerResponse):
    INDEX = 8  # type: int
    value: st.uint64


@dataclass(frozen=True)
class AIServerResponse__CreateIndex(AIServerResponse):
    INDEX = 9  # type: int
    value: st.uint64


AIServerResponse.VARIANTS = [
    AIServerResponse__Unit,
    AIServerResponse__Pong,
    AIServerResponse__ClientList,
    AIServerResponse__StoreList,
    AIServerResponse__InfoServer,
    AIServerResponse__Set,
    AIServerResponse__Get,
    AIServerResponse__GetSimN,
    AIServerResponse__Del,
    AIServerResponse__CreateIndex,
]


@dataclass(frozen=True)
class AIServerResult:
    results: typing.Sequence["Result"]

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, AIServerResult)

    @staticmethod
    def bincode_deserialize(input: bytes) -> "AIServerResult":
        v, buffer = bincode.deserialize(input, AIServerResult)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read")
        return v


@dataclass(frozen=True)
class AIStoreInfo:
    name: str
    query_model: "AIModel"
    index_model: "AIModel"
    embedding_size: st.uint64

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, AIStoreInfo)

    @staticmethod
    def bincode_deserialize(input: bytes) -> "AIStoreInfo":
        v, buffer = bincode.deserialize(input, AIStoreInfo)
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


@dataclass(frozen=True)
class ConnectedClient:
    address: str
    time_connected: "SystemTime"

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, ConnectedClient)

    @staticmethod
    def bincode_deserialize(input: bytes) -> "ConnectedClient":
        v, buffer = bincode.deserialize(input, ConnectedClient)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read")
        return v


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


class Result:
    VARIANTS = []  # type: typing.Sequence[typing.Type[Result]]

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, Result)

    @staticmethod
    def bincode_deserialize(input: bytes) -> "Result":
        v, buffer = bincode.deserialize(input, Result)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read")
        return v


@dataclass(frozen=True)
class Result__Ok(Result):
    INDEX = 0  # type: int
    value: "AIServerResponse"


@dataclass(frozen=True)
class Result__Err(Result):
    INDEX = 1  # type: int
    value: str


Result.VARIANTS = [
    Result__Ok,
    Result__Err,
]


@dataclass(frozen=True)
class ServerInfo:
    address: str
    version: "Version"
    type: "ServerType"
    limit: st.uint64
    remaining: st.uint64

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, ServerInfo)

    @staticmethod
    def bincode_deserialize(input: bytes) -> "ServerInfo":
        v, buffer = bincode.deserialize(input, ServerInfo)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read")
        return v


class ServerType:
    VARIANTS = []  # type: typing.Sequence[typing.Type[ServerType]]

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, ServerType)

    @staticmethod
    def bincode_deserialize(input: bytes) -> "ServerType":
        v, buffer = bincode.deserialize(input, ServerType)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read")
        return v


@dataclass(frozen=True)
class ServerType__Database(ServerType):
    INDEX = 0  # type: int
    pass


@dataclass(frozen=True)
class ServerType__AI(ServerType):
    INDEX = 1  # type: int
    pass


ServerType.VARIANTS = [
    ServerType__Database,
    ServerType__AI,
]


@dataclass(frozen=True)
class Similarity:
    value: st.float32

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, Similarity)

    @staticmethod
    def bincode_deserialize(input: bytes) -> "Similarity":
        v, buffer = bincode.deserialize(input, Similarity)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read")
        return v


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


@dataclass(frozen=True)
class StoreUpsert:
    inserted: st.uint64
    updated: st.uint64

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, StoreUpsert)

    @staticmethod
    def bincode_deserialize(input: bytes) -> "StoreUpsert":
        v, buffer = bincode.deserialize(input, StoreUpsert)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read")
        return v


@dataclass(frozen=True)
class SystemTime:
    secs_since_epoch: st.uint64
    nanos_since_epoch: st.uint32

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, SystemTime)

    @staticmethod
    def bincode_deserialize(input: bytes) -> "SystemTime":
        v, buffer = bincode.deserialize(input, SystemTime)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read")
        return v


@dataclass(frozen=True)
class Version:
    major: st.uint8
    minor: st.uint16
    patch: st.uint16

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, Version)

    @staticmethod
    def bincode_deserialize(input: bytes) -> "Version":
        v, buffer = bincode.deserialize(input, Version)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read")
        return v
