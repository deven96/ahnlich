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
    ahnlich_protocol = protocol.AhnlichProtocol(address="127.0.0.1", port=1369)
    ahnlich_protocol.send(message=ping)
    response = ahnlich_protocol.receive()
    ahnlich_protocol.client.close()
    print(response)


def test_client_sends_ping_success(base_protocol):
    db_client = AhnlichDBClient(client=base_protocol)
    response: server_response.ServerResult = db_client.ping()

    assert len(response.results) == 1
    assert response.results[0] == server_response.Result__Ok(
        server_response.ServerResponse__Pong()
    )
