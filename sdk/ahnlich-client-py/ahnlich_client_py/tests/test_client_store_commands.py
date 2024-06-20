from ahnlich_client_py.client import AhnlichDBClient
from ahnlich_client_py.internals import protocol, query, server_response
from ahnlich_client_py.libs import create_store_key

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
    assert isinstance(response.results[0], server_response.Result__Ok)

    store_lists: server_response.ServerResponse__StoreList = response.results[0].value
    assert len(store_lists.value) == 2
    store_info: server_response.StoreInfo = store_lists.value[0]
    queried_store_names = []
    for store_info in store_lists.value:
        queried_store_names.append(store_info.name)
    assert store_payload_with_predicates["store_name"] in queried_store_names


def test_client_set_in_store_succeeds(
    module_scopped_ahnlich_db, store_key, store_value
):
    port = module_scopped_ahnlich_db
    test_protocol = protocol.AhnlichProtocol(address="127.0.0.1", port=port)
    db_client = AhnlichDBClient(client=test_protocol)
    store_key_2 = create_store_key(data=[5.0, 3.0, 4.0, 3.9, 4.9])

    # prepare data
    store_data = {
        "store_name": store_payload_no_predicates["store_name"],
        "inputs": [(store_key, store_value), (store_key_2, {"rank": "chunin"})],
    }
    # process data
    response: server_response.ServerResult = db_client.set(**store_data)

    assert isinstance(response.results[0], server_response.Result__Ok)

    assert response.results[0] == server_response.Result__Ok(
        server_response.ServerResponse__Set(
            server_response.StoreUpsert(inserted=2, updated=0)
        )
    )


def test_client_get_key_succeeds(module_scopped_ahnlich_db, store_key, store_value):
    port = module_scopped_ahnlich_db
    test_protocol = protocol.AhnlichProtocol(address="127.0.0.1", port=port)
    db_client = AhnlichDBClient(client=test_protocol)

    # prepare data
    get_key_data = {
        "store_name": store_payload_no_predicates["store_name"],
        "keys": [store_key],
    }
    # process data
    response: server_response.ServerResult = db_client.get_key(**get_key_data)
    assert isinstance(response.results[0], server_response.Result__Ok)
    expected_result = [(store_key, store_value)]
    actual_response = response.results[0].value.value
    assert len(actual_response) == len(expected_result)
    assert actual_response[0][0].data == store_key.data


def test_client_get_predicate_fails_no_index_found_in_store(
    module_scopped_ahnlich_db, store_key, store_value
):
    port = module_scopped_ahnlich_db
    test_protocol = protocol.AhnlichProtocol(address="127.0.0.1", port=port)
    db_client = AhnlichDBClient(client=test_protocol)

    # prepare data
    get_predicate_data = {
        "store_name": store_payload_no_predicates["store_name"],
        "condition": query.PredicateCondition__Value(
            query.Predicate__Equals(key="job", value="sorcerer")
        ),
    }
    # process data
    response: server_response.ServerResult = db_client.get_predicate(
        **get_predicate_data
    )
    assert isinstance(response.results[0], server_response.Result__Err)

    error_message = "Predicate job not found in store"
    error_response = response.results[0].value
    assert error_message.lower() in error_response.lower()


def test_client_create_index_succeeds(module_scopped_ahnlich_db):
    port = module_scopped_ahnlich_db
    test_protocol = protocol.AhnlichProtocol(address="127.0.0.1", port=port)
    db_client = AhnlichDBClient(client=test_protocol)

    create_index_data = {
        "store_name": store_payload_no_predicates["store_name"],
        "predicates": ["job", "rank"],
    }
    response: server_response.ServerResult = db_client.create_index(**create_index_data)
    assert response.results[0] == server_response.Result__Ok(
        server_response.ServerResponse__CreateIndex(2)
    )


def test_client_get_predicate_succeeds(
    module_scopped_ahnlich_db, store_key, store_value
):
    port = module_scopped_ahnlich_db
    test_protocol = protocol.AhnlichProtocol(address="127.0.0.1", port=port)
    db_client = AhnlichDBClient(client=test_protocol)

    # prepare data
    get_predicate_data = {
        "store_name": store_payload_no_predicates["store_name"],
        "condition": query.PredicateCondition__Value(
            query.Predicate__Equals(key="job", value="sorcerer")
        ),
    }
    # process data
    response: server_response.ServerResult = db_client.get_predicate(
        **get_predicate_data
    )
    assert isinstance(response.results[0], server_response.Result__Ok)
    expected_result = [(store_key, store_value)]
    actual_response = response.results[0].value.value
    assert len(actual_response) == len(expected_result)
    assert actual_response[0][0].data == store_key.data


def test_client_get_sim_n_succeeds(module_scopped_ahnlich_db, store_key, store_value):
    port = module_scopped_ahnlich_db
    test_protocol = protocol.AhnlichProtocol(address="127.0.0.1", port=port)
    db_client = AhnlichDBClient(client=test_protocol)

    # closest to 1.0,2.0,3.0,4.0,5.0
    search_input = create_store_key(data=[1.0, 2.0, 3.0, 3.9, 4.9])

    # prepare data
    get_sim_n_data = {
        "store_name": store_payload_no_predicates["store_name"],
        "closest_n": 1,
        "search_input": search_input,
        "algorithm": query.Algorithm__CosineSimilarity(),
    }
    # process data
    response: server_response.ServerResult = db_client.get_sim_n(**get_sim_n_data)

    actual_results: server_response.ServerResponse__GetSimN = response.results[
        0
    ].value.value

    expected_results = [
        store_key,
        store_value,
        0.9999504,
    ]
    assert len(actual_results) == 1
    assert actual_results[0][1] == expected_results[1]
    assert str(expected_results[2]) in str(actual_results[0]).lower()


def test_client_drop_index_succeeds(module_scopped_ahnlich_db):
    port = module_scopped_ahnlich_db
    test_protocol = protocol.AhnlichProtocol(address="127.0.0.1", port=port)
    db_client = AhnlichDBClient(client=test_protocol)

    create_index_data = {
        "store_name": store_payload_no_predicates["store_name"],
        "predicates": ["to_drop"],
    }
    response: server_response.ServerResult = db_client.create_index(**create_index_data)
    assert response.results[0] == server_response.Result__Ok(
        server_response.ServerResponse__CreateIndex(1)
    )

    drop_index_data = {
        "store_name": store_payload_no_predicates["store_name"],
        "predicates": ["to_drop"],
        "error_if_not_exists": True,
    }

    response: server_response.ServerResult = db_client.drop_index(**drop_index_data)
    assert response.results[0] == server_response.Result__Ok(
        server_response.ServerResponse__Del(1)
    )


def test_client_delete_predicate_succeeds(module_scopped_ahnlich_db):
    port = module_scopped_ahnlich_db
    test_protocol = protocol.AhnlichProtocol(address="127.0.0.1", port=port)
    db_client = AhnlichDBClient(client=test_protocol)

    delete_predicate_data = {
        "store_name": store_payload_no_predicates["store_name"],
        "condition": query.PredicateCondition__Value(
            query.Predicate__Equals(key="rank", value="chunin")
        ),
    }

    response: server_response.ServerResult = db_client.delete_predicate(
        **delete_predicate_data
    )
    assert response.results[0] == server_response.Result__Ok(
        server_response.ServerResponse__Del(1)
    )


def test_client_delete_key_succeeds(module_scopped_ahnlich_db, store_key):
    port = module_scopped_ahnlich_db
    test_protocol = protocol.AhnlichProtocol(address="127.0.0.1", port=port)
    db_client = AhnlichDBClient(client=test_protocol)

    delete_key_data = {
        "store_name": store_payload_no_predicates["store_name"],
        "keys": [store_key],
    }

    response: server_response.ServerResult = db_client.delete_key(**delete_key_data)
    assert response.results[0] == server_response.Result__Ok(
        server_response.ServerResponse__Del(1)
    )


def test_client_drop_store_succeeds(module_scopped_ahnlich_db):
    port = module_scopped_ahnlich_db
    test_protocol = protocol.AhnlichProtocol(address="127.0.0.1", port=port)
    db_client = AhnlichDBClient(client=test_protocol)

    drop_store_data = {
        "store_name": store_payload_no_predicates["store_name"],
        "error_if_not_exists": True,
    }

    response: server_response.ServerResult = db_client.drop_store(**drop_store_data)
    assert response.results[0] == server_response.Result__Ok(
        server_response.ServerResponse__Del(1)
    )


def test_client_list_stores_reflects_dropped_store(
    module_scopped_ahnlich_db,
):
    port = module_scopped_ahnlich_db
    test_protocol = protocol.AhnlichProtocol(address="127.0.0.1", port=port)
    db_client = AhnlichDBClient(client=test_protocol)
    response: server_response.ServerResult = db_client.list_stores()
    store_list: server_response.ServerResponse__StoreList = response.results[0].value
    assert len(store_list.value) == 1
    store_info: server_response.StoreInfo = store_list.value[0]
    assert store_info.name == store_payload_with_predicates["store_name"]
    assert isinstance(response.results[0], server_response.Result__Ok)
