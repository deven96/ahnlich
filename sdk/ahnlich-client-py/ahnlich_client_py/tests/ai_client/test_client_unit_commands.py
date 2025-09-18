import pytest
from grpclib.client import Channel

from ahnlich_client_py.grpc.ai import query as ai_query
from ahnlich_client_py.grpc.ai import server as ai_server
from ahnlich_client_py.grpc.server_types import ServerType
from ahnlich_client_py.grpc.services import ai_service


@pytest.mark.asyncio
async def test_client_sends_ping_to_aiproxy_success(spin_up_ahnlich_ai):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_ai)
    client = ai_service.AiServiceStub(channel)
    try:
        response = await client.ping(ai_query.Ping())
        assert isinstance(response, ai_server.Pong)
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_client_sends_info_server_to_aiproxy_success(spin_up_ahnlich_ai):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_ai)
    client = ai_service.AiServiceStub(channel)
    try:
        response = await client.info_server(ai_query.InfoServer())
        assert isinstance(response.info.version, str)
        assert response.info.type == ServerType.AI
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_client_sends_list_stores_to_fresh_aiproxy_succeeds(spin_up_ahnlich_ai):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_ai)
    client = ai_service.AiServiceStub(channel)
    try:
        response = await client.list_stores(ai_query.ListStores())
        assert len(response.stores) == 0
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_client_works_using_context_manager(spin_up_ahnlich_ai):
    async with Channel(host="127.0.0.1", port=spin_up_ahnlich_ai) as channel:
        client = ai_service.AiServiceStub(channel)
        response = await client.list_stores(ai_query.ListStores())
        assert isinstance(response, ai_server.StoreList)
        assert len(response.stores) == 0
