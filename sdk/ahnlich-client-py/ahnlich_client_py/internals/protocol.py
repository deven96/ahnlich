import re
import socket
import typing

from ahnlich_client_py import config
from ahnlich_client_py.exceptions import (
    AhnlichClientException,
    AhnlichProtocolException,
)
from ahnlich_client_py.internals import ai_query, ai_response, db_query, db_response


class AhnlichMessageProtocol:
    def __init__(self, sock_timeout_sec: float = 5.0):
        self.version = self.get_version()
        self.timeout_sec = sock_timeout_sec

    def serialize_query(
        self, server_query: typing.Union[db_query.ServerQuery, ai_query.AIServerQuery]
    ) -> bytes:
        version = self.version.bincode_serialize()
        response = server_query.bincode_serialize()
        response_length = int(len(response)).to_bytes(8, "little")
        return config.HEADER + version + response_length + response

    def deserialize_server_response(
        self,
        b: bytes,
        response_class: typing.Union[
            db_response.ServerResult, ai_response.AIServerResult
        ],
    ) -> typing.Union[db_response.ServerResult, ai_response.AIServerResult]:
        return response_class([]).bincode_deserialize(b)

    def send(
        self,
        conn: socket.socket,
        message: typing.Union[db_query.ServerQuery, ai_query.AIServerQuery],
    ):
        serialized_bin = self.serialize_query(message)
        conn.sendall(serialized_bin)

    def receive(
        self,
        conn: socket.socket,
        response_class: typing.Union[
            db_response.ServerResult, ai_response.AIServerResult
        ],
    ) -> typing.Union[db_response.ServerResult, ai_response.AIServerResult]:
        conn.settimeout(self.timeout_sec)
        header = conn.recv(8)
        if header == b"":
            raise AhnlichProtocolException("socket connection broken")

        if header != config.HEADER:
            raise AhnlichProtocolException("Fake server")
        # ignore version of 5 bytes
        _version = conn.recv(5)
        length = conn.recv(8)
        # header length u64, little endian
        length_to_read = int.from_bytes(length, byteorder="little")
        # information data
        data = conn.recv(length_to_read)
        response = self.deserialize_server_response(data, response_class=response_class)
        return response

    @staticmethod
    def get_version() -> db_response.Version:
        with open(config.BASE_DIR / "VERSION", "r") as f:
            content = f.read()
            match = re.search('PROTOCOL="([^"]+)"', content)
            if not match:
                raise AhnlichClientException("Unable to Parse Protocol Version")
            str_version: str = match.group(1)
            # split and convert from str to int
            return db_response.Version(*map(lambda x: int(x), str_version.split(".")))
