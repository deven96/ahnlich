import typing

from ahnlich_client_py.builders import AhnlichAIRequestBuilder
from ahnlich_client_py.internals import ai_query, ai_response
from ahnlich_client_py.internals.async_base_client import BaseClient


class AsyncAhnlichAIRequestBuilder(AhnlichAIRequestBuilder):
    def __init__(self, tracing_id: str = None, client: BaseClient = None) -> None:
        self.queries: typing.List[ai_query.AIQuery] = []
        self.tracing_id = tracing_id
        self.client: BaseClient = client

    async def exec(self) -> ai_response.AIServerResult:
        """Executes a pipelined request"""
        return await self.client.process_request(message=self.to_server_query())
