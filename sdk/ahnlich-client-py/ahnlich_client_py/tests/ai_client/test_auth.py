import pytest
from grpclib.client import Channel
from grpclib.const import Status
from grpclib.exceptions import GRPCError

from ahnlich_client_py.grpc import ai
from ahnlich_client_py.grpc.services import ai_service
from ahnlich_client_py.tests.conftest import auth_metadata, make_client_ssl_context


@pytest.mark.asyncio
async def test_ai_unauthenticated_request_rejected(spin_up_ahnlich_ai_with_auth):
    port, cert_path = spin_up_ahnlich_ai_with_auth
    ssl_ctx = make_client_ssl_context(cert_path)
    channel = Channel(host="127.0.0.1", port=port, ssl=ssl_ctx)
    service = ai_service.AiServiceStub(channel)
    try:
        with pytest.raises(GRPCError) as exc_info:
            await service.ping(ai.query.Ping())
        assert exc_info.value.status == Status.UNAUTHENTICATED
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_ai_wrong_credentials_rejected(spin_up_ahnlich_ai_with_auth):
    port, cert_path = spin_up_ahnlich_ai_with_auth
    ssl_ctx = make_client_ssl_context(cert_path)
    channel = Channel(host="127.0.0.1", port=port, ssl=ssl_ctx)
    service = ai_service.AiServiceStub(channel)
    try:
        with pytest.raises(GRPCError) as exc_info:
            await service.ping(
                ai.query.Ping(), metadata=auth_metadata("aiuser", "wrongpassword")
            )
        assert exc_info.value.status == Status.UNAUTHENTICATED
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_ai_valid_credentials_accepted(spin_up_ahnlich_ai_with_auth):
    port, cert_path = spin_up_ahnlich_ai_with_auth
    ssl_ctx = make_client_ssl_context(cert_path)
    channel = Channel(host="127.0.0.1", port=port, ssl=ssl_ctx)
    service = ai_service.AiServiceStub(channel)
    try:
        response = await service.ping(
            ai.query.Ping(), metadata=auth_metadata("aiuser", "aipassword")
        )
        assert response == ai.server.Pong()
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_ai_with_auth_list_stores_succeeds(spin_up_ahnlich_ai_with_auth):
    port, cert_path = spin_up_ahnlich_ai_with_auth
    ssl_ctx = make_client_ssl_context(cert_path)
    channel = Channel(host="127.0.0.1", port=port, ssl=ssl_ctx)
    service = ai_service.AiServiceStub(channel)
    try:
        response = await service.list_stores(
            ai.query.ListStores(), metadata=auth_metadata("aiuser", "aipassword")
        )
        assert isinstance(response, ai.server.StoreList)
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_ai_no_auth_server_accepts_requests_without_token(spin_up_ahnlich_ai):
    port = spin_up_ahnlich_ai
    channel = Channel(host="127.0.0.1", port=port)
    service = ai_service.AiServiceStub(channel)
    try:
        response = await service.ping(ai.query.Ping())
        assert response == ai.server.Pong()
    finally:
        channel.close()
