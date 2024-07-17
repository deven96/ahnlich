import re
import socket
from contextlib import _GeneratorContextManager
from ipaddress import IPv4Address

from generic_connection_pool.contrib.socket import TcpSocketConnectionManager
from generic_connection_pool.exceptions import ConnectionPoolClosedError
from generic_connection_pool.threading import ConnectionPool

from ahnlich_client_py import config
from ahnlich_client_py.config import AhnlichDBPoolSettings
from ahnlich_client_py.exceptions import (
    AhnlichClientException,
    AhnlichProtocolException,
)
from ahnlich_client_py.internals import db_query, db_response


class AhnlichProtocol:
    def __init__(
        self,
        address: str,
        port: int,
        timeout_sec: float = 5.0,
        pool_settings: AhnlichDBPoolSettings = AhnlichDBPoolSettings(),
    ):
        self.address = IPv4Address(address)
        self.port = port
        self.connection_pool = self.create_connection_pool(pool_settings)
        self.version = self.get_version()
        self.timeout_sec = timeout_sec

    def serialize_query(self, server_query: db_query.ServerQuery) -> bytes:
        version = self.version.bincode_serialize()
        response = server_query.bincode_serialize()
        response_length = int(len(response)).to_bytes(8, "little")
        return config.HEADER + version + response_length + response

    def deserialize_server_response(self, b: bytes) -> db_response.ServerResult:
        return db_response.ServerResult([]).bincode_deserialize(b)

    @property
    def connect_generator(self) -> _GeneratorContextManager[socket.socket]:
        return self.connection_pool.connection(
            endpoint=(self.address, self.port), timeout=self.timeout_sec
        )

    def send(self, message: db_query.ServerQuery):
        serialized_bin = self.serialize_query(message)
        with self.connect_generator as conn:
            conn.sendall(serialized_bin)

    def receive(self) -> db_response.ServerResult:
        with self.connect_generator as conn:
            header = conn.recv(8)
            if header == b"":
                self.connection_pool.close()
                raise AhnlichProtocolException("socket connection broken")

            if header != config.HEADER:
                raise AhnlichProtocolException("Fake server")
            # ignore version of 5 bytes
            _version = conn.recv(5)
            length = conn.recv(8)
            # header length u64, little endian
            length_to_read = int.from_bytes(length, byteorder="little")
            # information data
            conn.settimeout(self.timeout_sec)
            data = conn.recv(length_to_read)
            response = self.deserialize_server_response(data)
            return response

    def process_request(
        self, message: db_query.ServerQuery
    ) -> db_response.ServerResult:
        self.send(message=message)
        response = self.receive()
        return response

    def create_connection_pool(self, settings: AhnlichDBPoolSettings) -> ConnectionPool:
        return ConnectionPool(
            connection_manager=TcpSocketConnectionManager(),
            idle_timeout=settings.idle_timeout,
            max_lifetime=settings.max_lifetime,
            min_idle=settings.min_idle_connections,
            max_size=settings.max_pool_size,
            total_max_size=settings.max_pool_size,
            background_collector=settings.enable_background_collector,
            dispose_batch_size=settings.dispose_batch_size,
        )

    def cleanup(self):
        try:
            self.connection_pool.close()
        except ConnectionPoolClosedError:
            pass

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

    def __enter__(self):
        return self

    def __exit__(self, *exc):
        self.cleanup()
