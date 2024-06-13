import typing

import socket
import serde_types as st
import query
import server_response

HEADER = b"AHNLICH;"
BUFFER_SIZE = 1024

class BincodeDeAndSerializer:

    def __init__(self):
        super().__init__()


    def serialize_query(self, server_query: query.ServerQuery) -> bytes:

        version = server_response.Version(0,1,0).bincode_serialize()
        response =  server_query.bincode_serialize()
        response_length = int(len(response)).to_bytes(8, "little")
        return HEADER + version + response_length + response

    def deserialize_server_response(self, b: bytes) -> server_response.ServerResult:
        return server_response.ServerResult([]).bincode_deserialize(b)


def test_serialize():
    ping = query.ServerQuery(
        queries = [
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
    message = BincodeDeAndSerializer().serialize_query(ping)
    with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
        s.connect(("127.0.0.1", 1369))
        s.sendall(message)
        header = s.recv(8)
        if header != HEADER:
            exit("Fake server")
        # ignore version of 5 bytes
        version = s.recv(5)
        print(server_response.Version.bincode_deserialize(version))
        length = s.recv(8)
        # header length u64, little endian
        length_to_read = int.from_bytes(length, byteorder="little")
        # information data
        s.settimeout(5)
        data = s.recv(length_to_read)
        print(BincodeDeAndSerializer().deserialize_server_response(data))

