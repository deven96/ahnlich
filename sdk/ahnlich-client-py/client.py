import typing
import serde_types as st
from internals import protocol, query, server_response


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

    def execute_requests(self, client: protocol.AhnlichProtocol):
        response = client.process_request(message=self.to_server_query())
        return response


class AhnlichDBClient:
    """Wrapper for interacting with Ahnlich database or ai"""

    def __init__(
        self,
        client: protocol.AhnlichProtocol,
    ) -> None:
        self.client = client
        # would abstract this away eventually, but for now easy does it
        self.builder = AhnlichRequestBuilder()

    def get_key(self, store_name: str, keys: typing.Sequence[query.Array]):

        self.builder.get_key(store_name=store_name, keys=keys)
        self.client.process_request(self.builder.to_server_query())

    def get_predicate(self, store_name: str, condition: query.PredicateCondition):
        self.builder.get_predicate(store_name=store_name, condition=condition)

    def get_sim_n(
        self,
        store_name: str,
        search_input: query.Array,
        closest_n: st.uint64,
        algorithm: query.Algorithm,
        condition: query.PredicateCondition = None,
    ):
        self.builder.get_sim_n(
            store_name=store_name,
            search_input=search_input,
            closest_n=closest_n,
            algorithm=algorithm,
            condition=condition,
        )
        self.client.process_request(self.builder.to_server_query())

    def create_index(self, store_name: str, predicates: typing.Sequence[str]):
        self.builder.create_index(store_name=store_name, predicates=predicates)
        self.client.process_request(self.builder.to_server_query())

    def drop_index(
        self,
        store_name: str,
        predicates: typing.Sequence[str],
        error_if_not_exists: bool,
    ):
        self.builder.drop_index(
            store_name=store_name,
            predicates=predicates,
            error_if_not_exists=error_if_not_exists,
        )
        self.client.process_request(self.builder.to_server_query())

    def set(
        self,
        store_name,
        inputs: typing.Sequence[typing.Tuple[query.Array, typing.Dict[str, str]]],
    ):
        self.builder.set(store_name=store_name, inputs=inputs)
        self.client.process_request(self.builder.to_server_query())

    def delete_key(self, store_name: str, keys: typing.Sequence[query.Array]):
        self.builder.delete_key(store_name=store_name, keys=keys)
        self.client.process_request(self.builder.to_server_query())

    def delete_predicate(self, store_name: str, condition: query.PredicateCondition):
        self.builder.delete_predicate(store_name=store_name, condition=condition)
        self.client.process_request(self.builder.to_server_query())

    def drop_store(self, store_name: str, error_if_not_exists: bool):
        self.builder.drop_store(
            store_name=store_name, error_if_not_exists=error_if_not_exists
        )
        self.client.process_request(self.builder.to_server_query())

    def create_store(
        self,
        store_name: str,
        dimension: st.uint64,
        create_predicates: typing.Sequence[str] = [],
        error_if_exists: bool = True,
    ) -> server_response.ServerResult:

        self.builder.create_store(
            store_name=store_name,
            dimension=dimension,
            create_predicates=create_predicates,
            error_if_exists=error_if_exists,
        )
        message = self.builder.to_server_query()
        return self.client.process_request(message=message)

    def list_stores(self):
        self.builder.list_stores()
        self.client.process_request(self.builder.to_server_query())

    def info_server(self) -> server_response.ServerResult:
        self.builder.info_server()
        return self.client.process_request(
            message=self.builder.to_server_query(),
        )

    def list_clients(self) -> server_response.ServerResult:
        self.builder.list_clients()
        return self.client.process_request(
            message=self.builder.to_server_query(),
        )

    def ping(self) -> server_response.ServerResult:
        self.builder.ping()
        return self.client.process_request(message=self.builder.to_server_query())
