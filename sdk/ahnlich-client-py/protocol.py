import socket

import query
import server_response

HEADER = b"AHNLICH;"
BUFFER_SIZE = 1024


class AhnlichProtocol:
    def __init__(self, address: str, port: int):
        self.address = address
        self.port = port
        self.client = self.connect()

    def serialize_query(self, server_query: query.ServerQuery) -> bytes:
        version = server_response.Version(0, 1, 0).bincode_serialize()
        response = server_query.bincode_serialize()
        response_length = int(len(response)).to_bytes(8, "little")
        return HEADER + version + response_length + response

    def deserialize_server_response(self, b: bytes) -> server_response.ServerResult:
        return server_response.ServerResult([]).bincode_deserialize(b)

    def connect(self) -> socket.socket:
        tcp_socket = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
        tcp_socket.connect((self.address, self.port))
        return tcp_socket

    def send(self, message: query.ServerQuery):
        serialized_bin = self.serialize_query(message)
        self.client.sendall(serialized_bin)

    def receive(self):
        header = self.client.recv(8)
        if header == b'':
            self.client.close()
            raise RuntimeError("socket connection broken")

        if header != HEADER:
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





def test_serialize():
    ping = query.ServerQuery(
        queries=[
            query.Query__Ping(),
            query.Query__InfoServer(),
            query.Query__ListClients(),
            query.Query__CreateStore(
                store="First Store",
                dimension=5,
                create_predicates=[],
                error_if_exists=True,
            ),
        ],
    )
    ahnlich_protocol = AhnlichProtocol(address="127.0.0.1", port=1369)
    ahnlich_protocol.send(message=ping)
    response = ahnlich_protocol.receive()
    ahnlich_protocol.client.close()
    print(response)
