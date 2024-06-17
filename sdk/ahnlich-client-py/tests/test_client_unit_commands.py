from internals import query, protocol, server_response
from client import AhnlichDBClient


def test_serialize():
    ping = query.ServerQuery(
        queries=[
            query.Query__Ping(),
            query.Query__InfoServer(),
            query.Query__ListClients(),
            query.Query__CreateStore(
                store="First Store",
                dimension=5,
                create_predicates=[],
                error_if_exists=True,
            ),
        ],
    )


def test_client_sends_ping_to_db_success(base_protocol):
    db_client = AhnlichDBClient(client=base_protocol)
    response: server_response.ServerResult = db_client.ping()

    assert len(response.results) == 1
    assert response.results[0] == server_response.Result__Ok(
        server_response.ServerResponse__Pong()
    )

def test_client_sends_list_clients_to_db_success(base_protocol):
    db_client = AhnlichDBClient(client=base_protocol)
    response: server_response.ServerResult = db_client.list_clients()
    assert len(response.results) == 1

def test_client_sends_info_server_to_db_success(base_protocol):
    db_client = AhnlichDBClient(client=base_protocol)
    response: server_response.ServerResult = db_client.info_server()
    assert len(response.results) == 1
    info_server: server_response.ServerInfo = response.results[0].value
    assert info_server.value.version == db_client.client.version
    assert info_server.value.type == server_response.ServerType__Database()
