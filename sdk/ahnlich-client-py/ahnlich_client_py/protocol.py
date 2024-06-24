import socket
from ipaddress import IPv4Address

from generic_connection_pool.contrib.socket import TcpSocketConnectionManager
from generic_connection_pool.threading import ConnectionPool

from ahnlich_client_py.config import service_config
from ahnlich_client_py.exceptions import AhnlichProtocolException
from ahnlich_client_py.internals import query, server_response


class AhnlichProtocol:
    def __init__(self, address: str, port: int, timeout_sec: float = 5.0):
        self.address = IPv4Address(address)
        self.port = port
        self.connection_pool = self.create_connection_pool()
        self.version = self.get_version()
        self.timeout_sec = timeout_sec
        self.conn = self.connect()

    def serialize_query(self, server_query: query.ServerQuery) -> bytes:
        version = self.version.bincode_serialize()
        response = server_query.bincode_serialize()
        response_length = int(len(response)).to_bytes(8, "little")
        return service_config.HEADER + version + response_length + response

    def deserialize_server_response(self, b: bytes) -> server_response.ServerResult:
        return server_response.ServerResult([]).bincode_deserialize(b)

    def connect(self) -> socket.socket:
        with self.connection_pool.connection(
            endpoint=(self.address, self.port), timeout=self.timeout_sec
        ) as conn:
            return conn

    def send(self, message: query.ServerQuery):
        serialized_bin = self.serialize_query(message)
        self.conn.sendall(serialized_bin)

    def receive(self) -> server_response.ServerResult:
        header = self.conn.recv(8)
        if header == b"":
            self.connection_pool.close()
            raise AhnlichProtocolException("socket connection broken")

        if header != service_config.HEADER:
            raise AhnlichProtocolException("Fake server")
        # ignore version of 5 bytes
        _version = self.conn.recv(5)
        length = self.conn.recv(8)
        # header length u64, little endian
        length_to_read = int.from_bytes(length, byteorder="little")
        # information data
        self.conn.settimeout(self.timeout_sec)
        data = self.conn.recv(length_to_read)
        response = self.deserialize_server_response(data)
        return response

    def process_request(
        self, message: query.ServerQuery
    ) -> server_response.ServerResult:
        self.send(message=message)
        response = self.receive()
        return response

    def create_connection_pool(self) -> ConnectionPool:

        return ConnectionPool(
            connection_manager=TcpSocketConnectionManager(),
            idle_timeout=service_config.POOL_IDLE_TIMEOUT,
            max_lifetime=service_config.POOL_MAX_LIFETIME,
            min_idle=service_config.POOL_MIN_IDLE_CONNECTIONS,
            max_size=service_config.POOL_MAX_SIZE,
            total_max_size=service_config.POOL_MAX_SIZE,
            background_collector=service_config.POOL_ENABLE_BACKGROUND_COLLECTOR,
            dispose_batch_size=service_config.POOL_DISPOSE_BATCH_SIZE,
        )

    def close(self):
        """closes a socket connection"""
        self.conn.close()

    def cleanup(self):
        self.conn.close()
        self.connection_pool.close()

    @staticmethod
    def get_version() -> server_response.Version:
        from importlib import metadata

        try:
            str_version = metadata.version(service_config.PACKAGE_NAME)
        except metadata.PackageNotFoundError:
            import toml

            with open(service_config.BASE_DIR / "pyproject.toml", "r") as f:
                reader = toml.load(f)
                str_version = reader["tool"]["poetry"]["version"]

        # split and convert from str to int
        return server_response.Version(*map(lambda x: int(x), str_version.split(".")))
