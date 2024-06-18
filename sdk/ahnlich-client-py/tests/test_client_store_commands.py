from client import AhnlichDBClient
from internals import query, server_response, protocol

store_payload_no_predicates = {
    "store_name": "Diretnan Station",
    "dimension": 5,
    "error_if_exists": True,
}

store_payload_with_predicates = {
    "store_name": "Diretnan Predication",
    "dimension": 5,
    "error_if_exists": True,
    "create_predicates": ["is_tyrannical", "rank"],
}


def test_client_sends_create_stores_succeeds(module_scopped_ahnlich_db):
    port = module_scopped_ahnlich_db
    test_protocol = protocol.AhnlichProtocol(address="127.0.0.1", port=port)
    db_client = AhnlichDBClient(client=test_protocol)
    response: server_response.ServerResult = db_client.create_store(
        **store_payload_no_predicates
    )

    assert response.results[0] == server_response.Result__Ok(
        server_response.ServerResponse__Unit()
    )


def test_client_sends_list_stores_on_existing_database_succeeds(
    module_scopped_ahnlich_db,
):
    port = module_scopped_ahnlich_db
    test_protocol = protocol.AhnlichProtocol(address="127.0.0.1", port=port)
    db_client = AhnlichDBClient(client=test_protocol)
    response: server_response.ServerResult = db_client.list_stores()
    store_list: server_response.ServerResponse__StoreList = response.results[0].value
    store_info: server_response.StoreInfo = store_list.value[0]
    assert store_info.name == store_payload_no_predicates["store_name"]
    assert isinstance(response.results[0], server_response.Result__Ok)


def test_client_sends_create_stores_with_predicates_succeeds(module_scopped_ahnlich_db):
    port = module_scopped_ahnlich_db
    test_protocol = protocol.AhnlichProtocol(address="127.0.0.1", port=port)
    db_client = AhnlichDBClient(client=test_protocol)
    response: server_response.ServerResult = db_client.create_store(
        **store_payload_with_predicates
    )

    assert response.results[0] == server_response.Result__Ok(
        server_response.ServerResponse__Unit()
    )


def test_client_list_stores_finds_created_store_with_predicate(
    module_scopped_ahnlich_db,
):
    port = module_scopped_ahnlich_db
    test_protocol = protocol.AhnlichProtocol(address="127.0.0.1", port=port)
    db_client = AhnlichDBClient(client=test_protocol)
    response: server_response.ServerResult = db_client.list_stores()
    store_lists: server_response.ServerResponse__StoreList = response.results[0].value
    assert len(store_lists.value) == 2
    store_info: server_response.StoreInfo = store_lists.value[1]
    assert store_info.name == store_payload_no_predicates["store_name"]
    assert isinstance(response.results[0], server_response.Result__Ok)
