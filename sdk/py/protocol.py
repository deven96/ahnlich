import typing
import bincode

import serde_types as st
import query

class BincodeDeAndSerializer:

    def __init__(self):
        super().__init__()


    def serialize_query(self, query):

        response =  query.bincode_serialize()

    def deserialize_server_response(self):
        pass


def test_serialize():
    ping = query.ServerQuery(
        queries = [query.Query__Ping(), query.Query__InfoServer(), query.Query__ListClients()],
    )

    print(ping.bincode_serialize())
