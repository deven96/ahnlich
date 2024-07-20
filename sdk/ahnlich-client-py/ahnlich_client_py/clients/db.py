import typing

from ahnlich_client_py import builders
from ahnlich_client_py.config import AhnlichDBPoolSettings
from ahnlich_client_py.internals import db_query, db_response
from ahnlich_client_py.internals import serde_types as st
from ahnlich_client_py.internals.base_client import BaseClient


class AhnlichDBClient(BaseClient):
    """Wrapper for interacting with Ahnlich database"""

    def __init__(
        self,
        address: str,
        port: int,
        connect_timeout_sec: float = 5.0,
        pool_settings: AhnlichDBPoolSettings = AhnlichDBPoolSettings(),
    ) -> None:

        super().__init__(
            address=address,
            port=port,
            connect_timeout_sec=connect_timeout_sec,
            pool_settings=pool_settings,
        )

        self.builder = builders.AhnlichDBRequestBuilder()

    def get_response_class(self):
        return db_response.ServerResult

    def get_key(
        self, store_name: str, keys: typing.Sequence[db_query.Array]
    ) -> db_response.ServerResult:

        self.builder.get_key(store_name=store_name, keys=keys)
        return self.process_request(self.builder.to_server_query())

    def get_by_predicate(
        self, store_name: str, condition: db_query.PredicateCondition
    ) -> db_response.ServerResult:
        self.builder.get_by_predicate(store_name=store_name, condition=condition)
        return self.process_request(self.builder.to_server_query())

    def get_sim_n(
        self,
        store_name: str,
        search_input: db_query.Array,
        closest_n: st.uint64,
        algorithm: db_query.Algorithm,
        condition: db_query.PredicateCondition = None,
    ) -> db_response.ServerResult:
        self.builder.get_sim_n(
            store_name=store_name,
            search_input=search_input,
            closest_n=closest_n,
            algorithm=algorithm,
            condition=condition,
        )
        return self.process_request(self.builder.to_server_query())

    def create_pred_index(
        self, store_name: str, predicates: typing.Sequence[str]
    ) -> db_response.ServerResult:
        self.builder.create_pred_index(store_name=store_name, predicates=predicates)
        return self.process_request(self.builder.to_server_query())

    def drop_pred_index(
        self,
        store_name: str,
        predicates: typing.Sequence[str],
        error_if_not_exists: bool,
    ) -> db_response.ServerResult:
        self.builder.drop_pred_index(
            store_name=store_name,
            predicates=predicates,
            error_if_not_exists=error_if_not_exists,
        )
        return self.process_request(self.builder.to_server_query())

    def set(
        self,
        store_name: str,
        inputs: typing.Sequence[
            typing.Tuple[db_query.Array, typing.Dict[str, db_query.MetadataValue]]
        ],
    ) -> db_response.ServerResult:
        self.builder.set(store_name=store_name, inputs=inputs)
        return self.process_request(self.builder.to_server_query())

    def delete_key(
        self, store_name: str, keys: typing.Sequence[db_query.Array]
    ) -> db_response.ServerResult:
        self.builder.delete_key(store_name=store_name, keys=keys)
        return self.process_request(self.builder.to_server_query())

    def delete_predicate(
        self, store_name: str, condition: db_query.PredicateCondition
    ) -> db_response.ServerResult:
        self.builder.delete_predicate(store_name=store_name, condition=condition)
        return self.process_request(self.builder.to_server_query())

    def drop_store(
        self, store_name: str, error_if_not_exists: bool
    ) -> db_response.ServerResult:
        self.builder.drop_store(
            store_name=store_name, error_if_not_exists=error_if_not_exists
        )
        return self.process_request(self.builder.to_server_query())

    def create_store(
        self,
        store_name: str,
        dimension: st.uint64,
        create_predicates: typing.Sequence[str] = None,
        non_linear_indices: typing.Sequence[db_query.NonLinearAlgorithm] = None,
        error_if_exists: bool = True,
    ) -> db_response.ServerResult:

        self.builder.create_store(
            store_name=store_name,
            dimension=dimension,
            create_predicates=create_predicates,
            non_linear_indices=non_linear_indices,
            error_if_exists=error_if_exists,
        )
        message = self.builder.to_server_query()
        return self.process_request(message=message)

    def list_stores(self) -> db_response.ServerResult:
        self.builder.list_stores()
        return self.process_request(self.builder.to_server_query())

    def info_server(self) -> db_response.ServerResult:
        self.builder.info_server()
        return self.process_request(
            message=self.builder.to_server_query(),
        )

    def list_clients(self) -> db_response.ServerResult:
        self.builder.list_clients()
        return self.process_request(
            message=self.builder.to_server_query(),
        )

    def ping(self) -> db_response.ServerResult:
        self.builder.ping()
        return self.process_request(message=self.builder.to_server_query())

    def pipeline(self) -> builders.AhnlichDBRequestBuilder:
        """Gives you a request builder to create multple requests"""
        return self.builder

    def exec(self) -> db_response.ServerResult:
        """Executes a pipelined request"""
        return self.process_request(message=self.builder.to_server_query())
