from ahnlich_client_py import clients
from ahnlich_client_py.internals import db_response


def test_client_sends_bulk_unit_requests_to_db_succeeds(spin_up_ahnlich_db):
    port = spin_up_ahnlich_db
    db_client = clients.AhnlichDBClient(address="127.0.0.1", port=port)
    request_builder = db_client.pipeline()
    request_builder.ping()
    request_builder.info_server()
    request_builder.list_clients()
    request_builder.list_stores()

    try:
        response: db_response.ServerResult = request_builder.exec()
    except Exception as e:
        print(f"Exception {e}")
    finally:
        db_client.cleanup()

    assert len(response.results) == 4
    assert response.results[0] == db_response.Result__Ok(
        db_response.ServerResponse__Pong()
    )
    # assert info servers
    info_server: db_response.ServerInfo = response.results[1].value
    assert info_server.value.version == db_client.message_protocol.version
    assert info_server.value.type == db_response.ServerType__Database()

    # assert list_stores
    assert response.results[3] == db_response.Result__Ok(
        db_response.ServerResponse__StoreList([])
    )
