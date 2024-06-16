import typing
import serde_types as st
from internals import protocol, query, server_response
import config


class NonZeroSizeInteger:
    def __init__(self, num: st.uint64) -> None:

        if num <= 0:
            raise Exception("Ahnlich expects a Non zero value as integers")
        self.value = num


class AhnlichDBClient:
    """Wrapper for interacting with Ahnlich database or ai"""

    def __init__(self, client: protocol.AhnlichProtocol) -> None:
        self.client = client

    def create_store(
        self,
        store_name: str,
        dimension: st.uint64,
        create_predicates: typing.Sequence[str] = [],
        error_if_exists: bool = True,
    ) -> server_response.ServerResult:

        non_zero = NonZeroSizeInteger(num=dimension)
        req = query.ServerQuery(
            queries=[
                query.Query__CreateStore(
                    store=store_name,
                    dimension=non_zero.value,
                    create_predicates=create_predicates,
                    error_if_exists=error_if_exists,
                )
            ]
        )
        return self.client.process_request(req)

    def get_key(self):
        pass

    def get_predicate():
        pass

    def get_sim_n():
        pass

    def create_index():
        pass

    def drop_index():
        pass

    def set_value():
        pass

    def delete_key():
        pass

    def delete_predicate():
        pass

    def drop_store():
        pass

    def list_stores():
        pass

    def info_server(self) -> server_response.ServerResult:
        req = query.ServerQuery(queries=[query.Query__InfoServer()])
        return self.client.process_request(req)

    def list_clients(self) -> server_response.ServerResult:
        req = query.ServerQuery(queries=[query.Query__ListClients()])
        return self.client.process_request(req)

    def ping(self) -> server_response.ServerResult:
        req = query.ServerQuery(queries=[query.Query__Ping()])
        return self.client.process_request(req)
