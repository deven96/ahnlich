import pytest
from grpclib.client import Channel
from grpclib.const import Status
from grpclib.exceptions import GRPCError

from ahnlich_client_py.grpc import db
from ahnlich_client_py.grpc.services import db_service
from ahnlich_client_py.tests.conftest import auth_metadata, make_client_ssl_context


@pytest.mark.asyncio
async def test_db_unauthenticated_request_rejected(spin_up_ahnlich_db_with_auth):
    port, cert_path = spin_up_ahnlich_db_with_auth
    ssl_ctx = make_client_ssl_context(cert_path)
    channel = Channel(host="127.0.0.1", port=port, ssl=ssl_ctx)
    service = db_service.DbServiceStub(channel)
    try:
        with pytest.raises(GRPCError) as exc_info:
            await service.ping(db.query.Ping())
        assert exc_info.value.status == Status.UNAUTHENTICATED
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_db_wrong_credentials_rejected(spin_up_ahnlich_db_with_auth):
    port, cert_path = spin_up_ahnlich_db_with_auth
    ssl_ctx = make_client_ssl_context(cert_path)
    channel = Channel(host="127.0.0.1", port=port, ssl=ssl_ctx)
    service = db_service.DbServiceStub(channel)
    try:
        with pytest.raises(GRPCError) as exc_info:
            await service.ping(
                db.query.Ping(), metadata=auth_metadata("alice", "wrongpassword")
            )
        assert exc_info.value.status == Status.UNAUTHENTICATED
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_db_valid_credentials_accepted(spin_up_ahnlich_db_with_auth):
    port, cert_path = spin_up_ahnlich_db_with_auth
    ssl_ctx = make_client_ssl_context(cert_path)
    channel = Channel(host="127.0.0.1", port=port, ssl=ssl_ctx)
    service = db_service.DbServiceStub(channel)
    try:
        response = await service.ping(
            db.query.Ping(), metadata=auth_metadata("alice", "alicepass")
        )
        assert response == db.server.Pong()
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_db_multiple_users_auth(spin_up_ahnlich_db_with_auth):
    port, cert_path = spin_up_ahnlich_db_with_auth
    ssl_ctx = make_client_ssl_context(cert_path)
    channel = Channel(host="127.0.0.1", port=port, ssl=ssl_ctx)
    service = db_service.DbServiceStub(channel)
    try:
        response = await service.ping(
            db.query.Ping(), metadata=auth_metadata("alice", "alicepass")
        )
        assert response == db.server.Pong()

        response = await service.ping(
            db.query.Ping(), metadata=auth_metadata("bob", "bobspass1")
        )
        assert response == db.server.Pong()

        with pytest.raises(GRPCError) as exc_info:
            await service.ping(
                db.query.Ping(), metadata=auth_metadata("charlie", "charliepass")
            )
        assert exc_info.value.status == Status.UNAUTHENTICATED
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_db_no_auth_server_accepts_requests_without_token(spin_up_ahnlich_db):
    port = spin_up_ahnlich_db
    channel = Channel(host="127.0.0.1", port=port)
    service = db_service.DbServiceStub(channel)
    try:
        response = await service.ping(db.query.Ping())
        assert response == db.server.Pong()
    finally:
        channel.close()
