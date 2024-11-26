import typing

from ahnlich_client_py.builders import AhnlichDBRequestBuilder
from ahnlich_client_py.internals import db_query, db_response
from ahnlich_client_py.internals.async_base_client import BaseClient


class AsyncAhnlichDBRequestBuilder(AhnlichDBRequestBuilder):
    def __init__(self, tracing_id: str = None, client: BaseClient = None) -> None:
        self.queries: typing.List[db_query.Query] = []
        self.tracing_id = tracing_id
        self.client: BaseClient = client

    async def exec(self) -> db_response.ServerResult:
        """Executes a pipelined request"""
        return await self.client.process_request(message=self.to_server_query())
