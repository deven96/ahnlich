import pytest
from grpclib.client import Channel

from ahnlich_client_py.grpc import db, server_types
from ahnlich_client_py.grpc.services import db_service


@pytest.mark.asyncio
async def test_client_sends_ping_to_db_success(module_scopped_ahnlich_db):
    port = module_scopped_ahnlich_db
    channel = Channel(host="127.0.0.1", port=port)
    service = db_service.DbServiceStub(channel)
    try:
        response = await service.ping(db.query.Ping())
        assert response == db.server.Pong()
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_client_sends_list_clients_to_db_success(module_scopped_ahnlich_db):
    port = module_scopped_ahnlich_db
    channel = Channel(host="127.0.0.1", port=port)
    service = db_service.DbServiceStub(channel)
    try:
        response = await service.list_clients(db.query.ListClients())
        print(f"Connected clients: {response}")
        assert len(response.clients) == 1
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_client_sends_info_server_to_db_success(module_scopped_ahnlich_db):
    port = module_scopped_ahnlich_db
    channel = Channel(host="127.0.0.1", port=port)
    service = db_service.DbServiceStub(channel)
    try:
        response = await service.info_server(db.query.InfoServer())
        assert response.info.version == "0.0.0"
        assert response.info.type == server_types.ServerType.Database
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_client_sends_list_stores_to_fresh_database_succeeds(
    module_scopped_ahnlich_db,
):
    port = module_scopped_ahnlich_db
    channel = Channel(host="127.0.0.1", port=port)
    service = db_service.DbServiceStub(channel)
    try:
        response = await service.list_stores(db.query.ListStores())
        assert response == db.server.StoreList(stores=[])
    finally:
        channel.close()
