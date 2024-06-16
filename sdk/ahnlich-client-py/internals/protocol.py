import socket

from internals import query
from internals import server_response
import config


class AhnlichProtocol:
    def __init__(self, address: str, port: int):
        self.address = address
        self.port = port
        self.client = self.connect()
        self.version = self.get_version()

    def serialize_query(self, server_query: query.ServerQuery) -> bytes:
        version = self.version.bincode_serialize()
        response = server_query.bincode_serialize()
        response_length = int(len(response)).to_bytes(8, "little")
        return config.HEADER + version + response_length + response

    def deserialize_server_response(self, b: bytes) -> server_response.ServerResult:
        return server_response.ServerResult([]).bincode_deserialize(b)

    def connect(self) -> socket.socket:
        tcp_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        tcp_socket.connect((self.address, self.port))
        return tcp_socket

    def send(self, message: query.ServerQuery):
        serialized_bin = self.serialize_query(message)
        self.client.sendall(serialized_bin)

    def receive(self) -> server_response.ServerResult:
        header = self.client.recv(8)
        if header == b"":
            self.client.close()
            raise RuntimeError("socket connection broken")

        if header != config.HEADER:
            exit("Fake server")
        # ignore version of 5 bytes
        _version = self.client.recv(5)
        length = self.client.recv(8)
        # header length u64, little endian
        length_to_read = int.from_bytes(length, byteorder="little")
        # information data
        self.client.settimeout(5)
        data = self.client.recv(length_to_read)
        response = self.deserialize_server_response(data)
        return response

    def process_request(
        self, message: query.ServerQuery
    ) -> server_response.ServerResult:
        self.send(message=message)
        response = self.receive()
        return response

    @staticmethod
    def get_version() -> server_response.Version:
        from importlib import metadata

        try:
            str_version = metadata.version(config.PACKAGE_NAME)
        except metadata.PackageNotFoundError:
            import toml

            with open(config.BASE_DIR / "pyproject.toml", "r") as f:
                reader = toml.load(f)
                str_version = reader["tool"]["poetry"]["version"]

        # split and convert from str to int
        return server_response.Version(*map(lambda x: int(x), str_version.split(".")))
