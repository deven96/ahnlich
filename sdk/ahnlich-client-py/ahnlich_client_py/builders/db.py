import typing

import numpy as np

from ahnlich_client_py import exceptions as ah_exceptions
from ahnlich_client_py.internals import db_query
from ahnlich_client_py.internals import serde_types as st
from ahnlich_client_py.protocol import AhnlichProtocol


class NonZeroSizeInteger:
    def __init__(self, num: st.uint64) -> None:

        if num <= 0:
            raise ah_exceptions.AhnlichValidationError(
                "Ahnlich expects a Non zero value as integers"
            )
        self.value = num


class AhnlichDBRequestBuilder:
    def __init__(self) -> None:
        self.queries: typing.List[db_query.Query] = []

    def create_store(
        self,
        store_name: str,
        dimension: st.uint64,
        create_predicates: typing.Sequence[str] = [],
        non_linear_indices: typing.Sequence[db_query.NonLinearAlgorithm] = [],
        error_if_exists: bool = True,
    ):
        if not create_predicates:
            create_predicates = []

        non_zero = NonZeroSizeInteger(num=dimension)
        self.queries.append(
            db_query.Query__CreateStore(
                store=store_name,
                dimension=non_zero.value,
                create_predicates=create_predicates,
                non_linear_indices=non_linear_indices,
                error_if_exists=error_if_exists,
            )
        )

    def get_key(self, store_name: str, keys: typing.Sequence[db_query.Array]):

        self.queries.append(db_query.Query__GetKey(store=store_name, keys=keys))

    def get_by_predicate(self, store_name: str, condition: db_query.PredicateCondition):
        self.queries.append(db_query.Query__GetPred(store=store_name, condition=condition))

    def get_sim_n(
        self,
        store_name: str,
        search_input: db_query.Array,
        closest_n: st.uint64,
        algorithm: db_query.Algorithm,
        condition: db_query.PredicateCondition = None,
    ):
        nonzero = NonZeroSizeInteger(closest_n)
        self.queries.append(
            db_query.Query__GetSimN(
                store=store_name,
                search_input=search_input,
                closest_n=nonzero.value,
                algorithm=algorithm,
                condition=condition,
            )
        )

    def create_pred_index(self, store_name: str, predicates: typing.Sequence[str]):
        self.queries.append(
            db_query.Query__CreatePredIndex(store=store_name, predicates=predicates)
        )

    def drop_pred_index(
        self,
        store_name: str,
        predicates: typing.Sequence[str],
        error_if_not_exists: bool,
    ):
        self.queries.append(
            db_query.Query__DropPredIndex(
                store=store_name,
                predicates=predicates,
                error_if_not_exists=error_if_not_exists,
            )
        )

    def set(
        self,
        store_name,
        inputs: typing.Sequence[
            typing.Tuple[db_query.Array, typing.Dict[str, db_query.MetadataValue]]
        ],
    ):
        self.queries.append(db_query.Query__Set(store=store_name, inputs=inputs))

    def delete_key(self, store_name: str, keys: typing.Sequence[db_query.Array]):
        self.queries.append(db_query.Query__DelKey(store=store_name, keys=keys))

    def delete_predicate(self, store_name: str, condition: db_query.PredicateCondition):
        self.queries.append(db_query.Query__DelPred(store=store_name, condition=condition))

    def drop_store(self, store_name: str, error_if_not_exists: bool):
        self.queries.append(
            db_query.Query__DropStore(
                store=store_name, error_if_not_exists=error_if_not_exists
            )
        )

    def list_stores(self):
        self.queries.append(db_query.Query__ListStores())

    def info_server(self):
        self.queries.append(db_query.Query__InfoServer())

    def list_clients(self):
        self.queries.append(db_query.Query__ListClients())

    def ping(self):
        self.queries.append(db_query.Query__Ping())

    def drop(self):
        self.queries.clear()

    def to_server_query(self) -> db_query.ServerQuery:
        if not self.queries:
            raise ah_exceptions.AhnlichClientException(
                "Must have atleast one request to be processed"
            )
        # not optimal, but so far, recreating the list and dropping the internal store.
        # seems straight forward
        queries = self.queries[:]

        server_query = db_query.ServerQuery(queries=queries)
        self.drop()
        return server_query

    def execute_requests(self, protocol: AhnlichProtocol):
        response = protocol.process_request(message=self.to_server_query())
        return response
