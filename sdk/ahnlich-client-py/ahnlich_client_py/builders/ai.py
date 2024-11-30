import typing

from ahnlich_client_py import exceptions as ah_exceptions
from ahnlich_client_py.internals import ai_query, ai_response
from ahnlich_client_py.internals import serde_types as st
from ahnlich_client_py.internals.base_client import BaseClient
from ahnlich_client_py.libs import NonZeroSizeInteger


class AhnlichAIRequestBuilder:
    def __init__(self, tracing_id: str = None, client: BaseClient = None) -> None:
        self.queries: typing.List[ai_query.AIQuery] = []
        self.tracing_id = tracing_id
        self.client: BaseClient = client

    def create_store(
        self,
        store_name: str,
        query_model: ai_query.AIModel = ai_query.AIModel__AllMiniLML6V2,
        index_model: ai_query.AIModel = ai_query.AIModel__AllMiniLML6V2,
        predicates: typing.Sequence[str] = None,
        non_linear_indices: typing.Sequence[ai_query.NonLinearAlgorithm] = None,
        error_if_exists: bool = True,
        store_original: bool = True,
    ):
        if not non_linear_indices:
            non_linear_indices = []
        if not predicates:
            predicates = []

        self.queries.append(
            ai_query.AIQuery__CreateStore(
                store=store_name,
                query_model=query_model,
                index_model=index_model,
                predicates=predicates,
                non_linear_indices=non_linear_indices,
                error_if_exists=error_if_exists,
                store_original=store_original,
            )
        )

    def get_pred(self, store_name: str, condition: ai_query.PredicateCondition):
        self.queries.append(
            ai_query.AIQuery__GetPred(store=store_name, condition=condition)
        )

    def get_sim_n(
        self,
        store_name: str,
        search_input: ai_query.StoreInput,
        closest_n: st.uint64 = 1,
        algorithm: ai_query.Algorithm = ai_query.Algorithm__CosineSimilarity,
        condition: typing.Optional[ai_query.PredicateCondition] = None,
        preprocess_action: ai_query.PreprocessAction = ai_query.PreprocessAction__ModelPreprocessing,
    ):
        nonzero_n = NonZeroSizeInteger(closest_n)
        self.queries.append(
            ai_query.AIQuery__GetSimN(
                store=store_name,
                search_input=search_input,
                closest_n=nonzero_n.value,
                algorithm=algorithm,
                condition=condition,
                preprocess_action=preprocess_action,
            )
        )

    def create_pred_index(self, store_name: str, predicates: typing.Sequence[str]):
        self.queries.append(
            ai_query.AIQuery__CreatePredIndex(store=store_name, predicates=predicates)
        )

    def create_non_linear_algorithm_index(
        self,
        store_name: str,
        non_linear_indices: typing.Sequence["NonLinearAlgorithm"],
    ):
        self.queries.append(
            ai_query.AIQuery__CreateNonLinearAlgorithmIndex(
                store=store_name, non_linear_indices=non_linear_indices
            )
        )

    def drop_pred_index(
        self,
        store_name: str,
        predicates: typing.Sequence[str],
        error_if_not_exists: bool = True,
    ):
        self.queries.append(
            ai_query.AIQuery__DropPredIndex(
                store=store_name,
                predicates=predicates,
                error_if_not_exists=error_if_not_exists,
            )
        )

    def drop_non_linear_algorithm_index(
        self,
        store_name: str,
        non_linear_indices: typing.Sequence["NonLinearAlgorithm"],
        error_if_not_exists: bool = True,
    ):
        self.queries.append(
            ai_query.AIQuery__DropNonLinearAlgorithmIndex(
                store=store_name,
                non_linear_indices=non_linear_indices,
                error_if_not_exists=error_if_not_exists,
            )
        )

    def set(
        self,
        store_name: str,
        inputs: typing.Sequence[
            typing.Tuple[ai_query.StoreInput, typing.Dict[str, ai_query.MetadataValue]]
        ],
        preprocess_action: ai_query.PreprocessAction = ai_query.PreprocessAction__NoPreprocessing,
    ):
        self.queries.append(
            ai_query.AIQuery__Set(
                store=store_name,
                inputs=inputs,
                preprocess_action=preprocess_action,
            )
        )

    def del_key(self, store_name: str, key: ai_query.StoreInput):
        self.queries.append(ai_query.AIQuery__DelKey(store=store_name, key=key))

    def get_key(self, store_name: str, keys: typing.Sequence[ai_query.StoreInput]):
        self.queries.append(ai_query.AIQuery__GetKey(store=store_name, keys=keys))

    def drop_store(self, store_name: str, error_if_not_exists: bool = True):
        self.queries.append(
            ai_query.AIQuery__DropStore(
                store=store_name, error_if_not_exists=error_if_not_exists
            )
        )

    def purge_stores(self):
        self.queries.append(ai_query.AIQuery__PurgeStores())

    def info_server(self):
        self.queries.append(ai_query.AIQuery__InfoServer())

    def list_stores(self):
        self.queries.append(ai_query.AIQuery__ListStores())

    def list_clients(self):
        self.queries.append(ai_query.AIQuery__ListClients())

    def ping(self):
        self.queries.append(ai_query.AIQuery__Ping())

    def drop(self):
        self.queries.clear()

    def to_server_query(self) -> ai_query.AIServerQuery:
        if not self.queries:
            raise ah_exceptions.AhnlichClientException(
                "Must have atleast one ai request to be processed"
            )
        # not optimal, but so far, recreating the list and dropping the internal store.
        # seems straight forward
        queries = self.queries[:]
        server_query = ai_query.AIServerQuery(queries=queries, trace_id=self.tracing_id)
        self.drop()
        return server_query

    def exec(self) -> ai_response.AIServerResult:
        """Executes a pipelined request"""
        return self.client.process_request(message=self.to_server_query())
