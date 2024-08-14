# pyre-strict
import typing
from dataclasses import dataclass

from ahnlich_client_py.internals import bincode
from ahnlich_client_py.internals import serde_types as st


@dataclass(frozen=True)
class Array:
    v: st.uint8
    dim: typing.Tuple[st.uint64]
    data: typing.Sequence[st.float32]

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, Array)

    @staticmethod
    def bincode_deserialize(input: bytes) -> "Array":
        v, buffer = bincode.deserialize(input, Array)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read")
        return v


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
    value: "ServerResponse"


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


class ServerResponse:
    VARIANTS = []  # type: typing.Sequence[typing.Type[ServerResponse]]

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, ServerResponse)

    @staticmethod
    def bincode_deserialize(input: bytes) -> "ServerResponse":
        v, buffer = bincode.deserialize(input, ServerResponse)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read")
        return v


@dataclass(frozen=True)
class ServerResponse__Unit(ServerResponse):
    INDEX = 0  # type: int
    pass


@dataclass(frozen=True)
class ServerResponse__Pong(ServerResponse):
    INDEX = 1  # type: int
    pass


@dataclass(frozen=True)
class ServerResponse__ClientList(ServerResponse):
    INDEX = 2  # type: int
    value: typing.Sequence["ConnectedClient"]


@dataclass(frozen=True)
class ServerResponse__StoreList(ServerResponse):
    INDEX = 3  # type: int
    value: typing.Sequence["StoreInfo"]


@dataclass(frozen=True)
class ServerResponse__InfoServer(ServerResponse):
    INDEX = 4  # type: int
    value: "ServerInfo"


@dataclass(frozen=True)
class ServerResponse__Set(ServerResponse):
    INDEX = 5  # type: int
    value: "StoreUpsert"


@dataclass(frozen=True)
class ServerResponse__Get(ServerResponse):
    INDEX = 6  # type: int
    value: typing.Sequence[typing.Tuple["Array", typing.Dict[str, "MetadataValue"]]]


@dataclass(frozen=True)
class ServerResponse__GetSimN(ServerResponse):
    INDEX = 7  # type: int
    value: typing.Sequence[
        typing.Tuple["Array", typing.Dict[str, "MetadataValue"], "Similarity"]
    ]


@dataclass(frozen=True)
class ServerResponse__Del(ServerResponse):
    INDEX = 8  # type: int
    value: st.uint64


@dataclass(frozen=True)
class ServerResponse__CreateIndex(ServerResponse):
    INDEX = 9  # type: int
    value: st.uint64


ServerResponse.VARIANTS = [
    ServerResponse__Unit,
    ServerResponse__Pong,
    ServerResponse__ClientList,
    ServerResponse__StoreList,
    ServerResponse__InfoServer,
    ServerResponse__Set,
    ServerResponse__Get,
    ServerResponse__GetSimN,
    ServerResponse__Del,
    ServerResponse__CreateIndex,
]


@dataclass(frozen=True)
class ServerResult:
    results: typing.Sequence["Result"]

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, ServerResult)

    @staticmethod
    def bincode_deserialize(input: bytes) -> "ServerResult":
        v, buffer = bincode.deserialize(input, ServerResult)
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


@dataclass(frozen=True)
class StoreInfo:
    name: str
    len: st.uint64
    size_in_bytes: st.uint64

    def bincode_serialize(self) -> bytes:
        return bincode.serialize(self, StoreInfo)

    @staticmethod
    def bincode_deserialize(input: bytes) -> "StoreInfo":
        v, buffer = bincode.deserialize(input, StoreInfo)
        if buffer:
            raise st.DeserializationError("Some input bytes were not read")
        return v


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
