import typing
import bincode

import serde_types as st
import query

class BincodeDeAndSerializer:

    def __init__(self):
        super().__init__()


    def serialize_query(self, query):

        response =  query.bincode_serialize()
        print(response)

    def deserialize_server_response(self):
        pass


serializer = BincodeDeAndSerializer()


#result = serializer.serialize_query()
ping = query.Query__Ping()

print(ping.bincode_serialize())
