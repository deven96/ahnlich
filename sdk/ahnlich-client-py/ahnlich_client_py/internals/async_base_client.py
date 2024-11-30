import asyncio
import typing
from contextlib import _AsyncGeneratorContextManager
from ipaddress import IPv4Address

from generic_connection_pool.asyncio import ConnectionPool
from generic_connection_pool.contrib.socket_async import TcpStreamConnectionManager
from generic_connection_pool.exceptions import ConnectionPoolClosedError

from ahnlich_client_py.config import AhnlichPoolSettings
from ahnlich_client_py.exceptions import AhnlichClientException
from ahnlich_client_py.internals import (
    ai_query,
    ai_response,
    db_query,
    db_response,
    pool_wrapper,
    protocol,
)

Hostname = str
Port = int
Endpoint = typing.Tuple[Hostname, Port]
Connection = typing.Tuple[asyncio.StreamReader, asyncio.StreamWriter]


class BaseClient:
    def __init__(
        self,
        address: Hostname,
        port: Port,
        connect_timeout_sec: float = 5.0,
        pool_settings: AhnlichPoolSettings = AhnlichPoolSettings(),
    ) -> None:
        if address is None or port is None:
            raise AhnlichClientException("Address and port must be provided")
        self.address = address
        self.port = port
        self.connection_pool = self.create_connection_pool(pool_settings)
        self.timeout_sec = connect_timeout_sec
        self.message_protocol = protocol.AhnlichMessageProtocol(
            sock_timeout_sec=connect_timeout_sec
        )

    async def __aenter__(self):
        return self

    async def __aexit__(self, *exc):
        await self.cleanup()

    def create_connection_pool(
        self, settings: AhnlichPoolSettings
    ) -> ConnectionPool[Endpoint, Connection]:
        return ConnectionPool[Endpoint, Connection](
            TcpStreamConnectionManager(),
            idle_timeout=settings.idle_timeout,
            max_lifetime=settings.max_lifetime,
            min_idle=settings.min_idle_connections,
            max_size=settings.max_pool_size,
            total_max_size=settings.max_pool_size,
            background_collector=settings.enable_background_collector,
            dispose_batch_size=settings.dispose_batch_size,
        )

    @property
    def connected_socket(self) -> _AsyncGeneratorContextManager[Connection]:
        return self.connection_pool.connection(
            endpoint=(self.address, self.port), timeout=self.timeout_sec
        )

    async def cleanup(self):
        try:
            await self.connection_pool.close()
        except ConnectionPoolClosedError:
            pass

    async def process_request(
        self, message: typing.Union[db_query.ServerQuery, ai_query.AIServerQuery]
    ) -> typing.Union[db_response.ServerResult, ai_response.AIServerResult]:
        async with self.connected_socket as (stream_reader, stream_writer):
            _ = await self.message_protocol.async_send(
                writer=stream_writer, message=message
            )
            return await self.message_protocol.async_receive(
                reader=stream_reader, response_class=self.get_response_class()
            )

    def get_response_class(self):
        """Either the ai_response::AIServerResult or db_response::ServerResult class"""
        raise NotImplementedError()
