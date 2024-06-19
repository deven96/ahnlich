import typing

import numpy as np

import serde_types as st
from internals import query
from internals.protocol import AhnlichProtocol


class NonZeroSizeInteger:
    def __init__(self, num: st.uint64) -> None:

        if num <= 0:
            raise Exception("Ahnlich expects a Non zero value as integers")
        self.value = num


# TODO: might become a  class that both ahnlichai and db would use, without really changing the
# composition of the underlying clients
class AhnlichRequestBuilder:
    def __init__(self) -> None:
        self.queries: typing.List[query.Query] = []

    def create_store(
        self,
        store_name: str,
        dimension: st.uint64,
        create_predicates: typing.Sequence[str] = [],
        error_if_exists: bool = True,
    ):

        non_zero = NonZeroSizeInteger(num=dimension)
        self.queries.append(
            query.Query__CreateStore(
                store=store_name,
                dimension=non_zero.value,
                create_predicates=create_predicates,
                error_if_exists=error_if_exists,
            )
        )

    def get_key(self, store_name: str, keys: typing.Sequence[query.Array]):

        self.queries.append(query.Query__GetKey(store=store_name, keys=keys))

    def get_predicate(self, store_name: str, condition: query.PredicateCondition):
        self.queries.append(query.Query__GetPred(store=store_name, condition=condition))

    def get_sim_n(
        self,
        store_name: str,
        search_input: query.Array,
        closest_n: st.uint64,
        algorithm: query.Algorithm,
        condition: query.PredicateCondition = None,
    ):
        nonzero = NonZeroSizeInteger(closest_n)
        self.queries.append(
            query.Query__GetSimN(
                store=store_name,
                search_input=search_input,
                closest_n=nonzero.value,
                algorithm=algorithm,
                condition=condition,
            )
        )

    def create_index(self, store_name: str, predicates: typing.Sequence[str]):
        self.queries.append(
            query.Query__CreateIndex(store=store_name, predicates=predicates)
        )

    def drop_index(
        self,
        store_name: str,
        predicates: typing.Sequence[str],
        error_if_not_exists: bool,
    ):
        self.queries.append(
            query.Query__DropIndex(
                store=store_name,
                predicates=predicates,
                error_if_not_exists=error_if_not_exists,
            )
        )

    def set(
        self,
        store_name,
        inputs: typing.Sequence[typing.Tuple[query.Array, typing.Dict[str, str]]],
    ):
        self.queries.append(query.Query__Set(store=store_name, inputs=inputs))

    def delete_key(self, store_name: str, keys: typing.Sequence[query.Array]):
        self.queries.append(query.Query__DelKey(store=store_name, keys=keys))

    def delete_predicate(self, store_name: str, condition: query.PredicateCondition):
        self.queries.append(query.Query__DelPred(store=store_name, condition=condition))

    def drop_store(self, store_name: str, error_if_not_exists: bool):
        self.queries.append(
            query.Query__DropStore(
                store=store_name, error_if_not_exists=error_if_not_exists
            )
        )

    def list_stores(self):
        self.queries.append(query.Query__ListStores())

    def info_server(self):
        self.queries.append(query.Query__InfoServer())

    def list_clients(self):
        self.queries.append(query.Query__ListClients())

    def ping(self):
        self.queries.append(query.Query__Ping())

    def drop(self):
        self.queries.clear()

    def to_server_query(self) -> query.ServerQuery:
        if not self.queries:
            raise Exception("Must have atleast one request to be processed")
        # not optimal, but so far, recreating the list and dropping the internal store.
        # seems straight forward
        queries = self.queries[:]

        server_query = query.ServerQuery(queries=queries)
        self.drop()
        return server_query

    def execute_requests(self, client: AhnlichProtocol):
        response = client.process_request(message=self.to_server_query())
        return response


def create_store_key(data: typing.List[float], v: int = 1) -> query.Array:
    np_array = np.array(data, dtype=np.float32)
    dimensions = (st.uint64(np_array.shape[0]),)
    store_key = query.Array(v=st.uint8(v), dim=dimensions, data=np_array.tolist())
    return store_key
