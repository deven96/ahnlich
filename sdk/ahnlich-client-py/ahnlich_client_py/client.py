import typing

from ahnlich_client_py import builders, protocol
from ahnlich_client_py.internals import query
from ahnlich_client_py.internals import serde_types as st
from ahnlich_client_py.internals import server_response


class AhnlichDBClient:
    """Wrapper for interacting with Ahnlich database or ai"""

    def __init__(self, address: str, port: int, timeout_sec: float = 5.0) -> None:
        self.protocol = protocol.AhnlichProtocol(
            address=address, port=port, timeout_sec=timeout_sec
        )
        # would abstract this away eventually, but for now easy does it
        self.builder = builders.AhnlichDBRequestBuilder()

    def get_key(
        self, store_name: str, keys: typing.Sequence[query.Array]
    ) -> server_response.ServerResult:

        self.builder.get_key(store_name=store_name, keys=keys)
        return self.protocol.process_request(self.builder.to_server_query())

    def get_by_predicate(
        self, store_name: str, condition: query.PredicateCondition
    ) -> server_response.ServerResult:
        self.builder.get_by_predicate(store_name=store_name, condition=condition)
        return self.protocol.process_request(self.builder.to_server_query())

    def get_sim_n(
        self,
        store_name: str,
        search_input: query.Array,
        closest_n: st.uint64,
        algorithm: query.Algorithm,
        condition: query.PredicateCondition = None,
    ) -> server_response.ServerResult:
        self.builder.get_sim_n(
            store_name=store_name,
            search_input=search_input,
            closest_n=closest_n,
            algorithm=algorithm,
            condition=condition,
        )
        return self.protocol.process_request(self.builder.to_server_query())

    def create_index(
        self, store_name: str, predicates: typing.Sequence[str]
    ) -> server_response.ServerResult:
        self.builder.create_index(store_name=store_name, predicates=predicates)
        return self.protocol.process_request(self.builder.to_server_query())

    def drop_index(
        self,
        store_name: str,
        predicates: typing.Sequence[str],
        error_if_not_exists: bool,
    ) -> server_response.ServerResult:
        self.builder.drop_index(
            store_name=store_name,
            predicates=predicates,
            error_if_not_exists=error_if_not_exists,
        )
        return self.protocol.process_request(self.builder.to_server_query())

    def set(
        self,
        store_name: str,
        inputs: typing.Sequence[typing.Tuple[query.Array, typing.Dict[str, str]]],
    ) -> server_response.ServerResult:
        self.builder.set(store_name=store_name, inputs=inputs)
        return self.protocol.process_request(self.builder.to_server_query())

    def delete_key(
        self, store_name: str, keys: typing.Sequence[query.Array]
    ) -> server_response.ServerResult:
        self.builder.delete_key(store_name=store_name, keys=keys)
        return self.protocol.process_request(self.builder.to_server_query())

    def delete_predicate(
        self, store_name: str, condition: query.PredicateCondition
    ) -> server_response.ServerResult:
        self.builder.delete_predicate(store_name=store_name, condition=condition)
        return self.protocol.process_request(self.builder.to_server_query())

    def drop_store(
        self, store_name: str, error_if_not_exists: bool
    ) -> server_response.ServerResult:
        self.builder.drop_store(
            store_name=store_name, error_if_not_exists=error_if_not_exists
        )
        return self.protocol.process_request(self.builder.to_server_query())

    def create_store(
        self,
        store_name: str,
        dimension: st.uint64,
        create_predicates: typing.Sequence[str] = None,
        error_if_exists: bool = True,
    ) -> server_response.ServerResult:
        if not create_predicates:
            create_predicates = []
        self.builder.create_store(
            store_name=store_name,
            dimension=dimension,
            create_predicates=create_predicates,
            error_if_exists=error_if_exists,
        )
        message = self.builder.to_server_query()
        return self.protocol.process_request(message=message)

    def list_stores(self) -> server_response.ServerResult:
        self.builder.list_stores()
        return self.protocol.process_request(self.builder.to_server_query())

    def info_server(self) -> server_response.ServerResult:
        self.builder.info_server()
        return self.protocol.process_request(
            message=self.builder.to_server_query(),
        )

    def list_clients(self) -> server_response.ServerResult:
        self.builder.list_clients()
        return self.protocol.process_request(
            message=self.builder.to_server_query(),
        )

    def ping(self) -> server_response.ServerResult:
        self.builder.ping()
        return self.protocol.process_request(message=self.builder.to_server_query())

    def pipeline(self) -> builders.AhnlichDBRequestBuilder:
        """Gives you a request builder to create multple requests"""
        return self.builder

    def exec(self) -> server_response.ServerResult:
        """Executes a pipelined request"""
        return self.protocol.process_request(message=self.builder.to_server_query())

    def close(self):
        """closes the socket connection"""
        self.protocol.close()

    def cleanup(self):
        """closes the socket connection as well as connection pool"""
        self.close()
        self.protocol.cleanup()
