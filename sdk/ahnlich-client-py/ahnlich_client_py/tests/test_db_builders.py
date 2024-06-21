from ahnlich_client_py import protocol
from ahnlich_client_py.builders import AhnlichDBRequestBuilder
from ahnlich_client_py.internals import server_response


def test_client_sends_bulk_unit_requests_to_db_succeeds(spin_up_ahnlich_db):
    port = spin_up_ahnlich_db
    proto = protocol.AhnlichProtocol(address="127.0.0.1", port=port)
    request_builder = AhnlichDBRequestBuilder()
    request_builder.ping()
    request_builder.info_server()
    request_builder.list_clients()
    request_builder.list_stores()

    response: server_response.ServerResult = request_builder.execute_requests(
        protocol=proto
    )

    assert len(response.results) == 4
    assert response.results[0] == server_response.Result__Ok(
        server_response.ServerResponse__Pong()
    )
    # assert info servers
    info_server: server_response.ServerInfo = response.results[1].value
    assert info_server.value.version == proto.version
    assert info_server.value.type == server_response.ServerType__Database()

    # assert list_stores
    assert response.results[3] == server_response.Result__Ok(
        server_response.ServerResponse__StoreList([])
    )
