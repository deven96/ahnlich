import socket
import typing
from contextlib import _GeneratorContextManager
from ipaddress import IPv4Address

from generic_connection_pool.exceptions import ConnectionPoolClosedError
from generic_connection_pool.threading import ConnectionPool

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


class BaseClient:
    def __init__(
        self,
        address: str,
        port: int,
        connect_timeout_sec: float = 5.0,
        pool_settings: AhnlichPoolSettings = AhnlichPoolSettings(),
    ) -> None:
        if address is None or port is None:
            raise AhnlichClientException("Address and port must be provided")
        self.address = IPv4Address(address)
        self.port = port
        self.connection_pool = self.create_connection_pool(pool_settings)
        self.timeout_sec = connect_timeout_sec
        self.message_protocol = protocol.AhnlichMessageProtocol(
            sock_timeout_sec=connect_timeout_sec
        )

    def __del__(self):
        self.cleanup()

    def __enter__(self):
        return self

    def __exit__(self, *exc):
        self.cleanup()

    def create_connection_pool(self, settings: AhnlichPoolSettings) -> ConnectionPool:
        return ConnectionPool(
            connection_manager=pool_wrapper.AhnlichTcpSocketConnectionManager(),
            idle_timeout=settings.idle_timeout,
            max_lifetime=settings.max_lifetime,
            min_idle=settings.min_idle_connections,
            max_size=settings.max_pool_size,
            total_max_size=settings.max_pool_size,
            background_collector=settings.enable_background_collector,
            dispose_batch_size=settings.dispose_batch_size,
        )

    @property
    def connected_socket(self) -> _GeneratorContextManager[socket.socket]:
        return self.connection_pool.connection(
            endpoint=(self.address, self.port), timeout=self.timeout_sec
        )

    def cleanup(self):
        try:
            self.connection_pool.close()
        except ConnectionPoolClosedError:
            pass

    def process_request(
        self, message: typing.Union[db_query.ServerQuery, ai_query.AIServerQuery]
    ) -> typing.Union[db_response.ServerResult, ai_response.AIServerResult]:
        with self.connected_socket as conn:
            self.message_protocol.send(conn=conn, message=message)
            response = self.message_protocol.receive(
                conn=conn, response_class=self.get_response_class()
            )
        return response

    def get_response_class(self):
        """Either the ai_response::AIServerResult or db_response::ServerResult class"""
        raise NotImplementedError()
