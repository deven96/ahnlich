from ahnlich_client_py import client
from ahnlich_client_py.internals import db_response


def test_client_sends_ping_to_db_success(db_client):
    response: db_response.ServerResult = db_client.ping()

    assert len(response.results) == 1
    assert response.results[0] == db_response.Result__Ok(
        db_response.ServerResponse__Pong()
    )


def test_client_sends_list_clients_to_db_success(db_client):

    response: db_response.ServerResult = db_client.list_clients()
    assert len(response.results) == 1


def test_client_sends_info_server_to_db_success(db_client):
    response: db_response.ServerResult = db_client.info_server()
    assert len(response.results) == 1
    info_server: db_response.ServerInfo = response.results[0].value
    assert info_server.value.version == db_client.message_protocol.version
    assert info_server.value.type == db_response.ServerType__Database()


def test_client_sends_list_stores_to_fresh_database_succeeds(spin_up_ahnlich_db):
    port = spin_up_ahnlich_db
    db_client = client.AhnlichDBClient(address="127.0.0.1", port=port)
    try:
        response: db_response.ServerResult = db_client.list_stores()
    finally:
        db_client.cleanup()
    assert response.results[0] == db_response.Result__Ok(
        db_response.ServerResponse__StoreList([])
    )


def test_client_works_using_protocol_in_context(spin_up_ahnlich_db):
    port = spin_up_ahnlich_db
    with client.AhnlichDBClient(address="127.0.0.1", port=port) as db_client:
        response: db_response.ServerResult = db_client.list_stores()
    assert response.results[0] == db_response.Result__Ok(
        db_response.ServerResponse__StoreList([])
    )
