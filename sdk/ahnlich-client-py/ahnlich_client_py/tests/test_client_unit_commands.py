from ahnlich_client_py import protocol
from ahnlich_client_py.client import AhnlichDBClient
from ahnlich_client_py.internals import server_response


def test_client_sends_ping_to_db_success(base_protocol):
    db_client = AhnlichDBClient(ahnlich_protocol=base_protocol)
    response: server_response.ServerResult = db_client.ping()

    assert len(response.results) == 1
    assert response.results[0] == server_response.Result__Ok(
        server_response.ServerResponse__Pong()
    )


def test_client_sends_list_clients_to_db_success(base_protocol):
    db_client = AhnlichDBClient(ahnlich_protocol=base_protocol)
    response: server_response.ServerResult = db_client.list_clients()
    assert len(response.results) == 1


def test_client_sends_info_server_to_db_success(base_protocol):
    db_client = AhnlichDBClient(ahnlich_protocol=base_protocol)
    response: server_response.ServerResult = db_client.info_server()
    assert len(response.results) == 1
    info_server: server_response.ServerInfo = response.results[0].value
    assert info_server.value.version == db_client.protocol.version
    assert info_server.value.type == server_response.ServerType__Database()


def test_client_sends_list_stores_to_fresh_database_succeeds(spin_up_ahnlich_db):
    port = spin_up_ahnlich_db
    test_protocol = protocol.AhnlichProtocol(address="127.0.0.1", port=port)
    db_client = AhnlichDBClient(ahnlich_protocol=test_protocol)
    response: server_response.ServerResult = db_client.list_stores()

    assert response.results[0] == server_response.Result__Ok(
        server_response.ServerResponse__StoreList([])
    )
