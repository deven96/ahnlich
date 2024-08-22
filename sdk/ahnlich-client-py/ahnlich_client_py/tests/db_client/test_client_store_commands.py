import typing

from ahnlich_client_py.clients import AhnlichDBClient
from ahnlich_client_py.internals import db_query, db_response
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


def test_client_sends_create_stores_succeeds(spin_up_ahnlich_db):
    port = spin_up_ahnlich_db

    db_client = AhnlichDBClient(address="127.0.0.1", port=port)
    try:
        response: db_response.ServerResult = db_client.create_store(
            **store_payload_no_predicates
        )
    finally:
        db_client.cleanup()

    assert response.results[0] == db_response.Result__Ok(
        db_response.ServerResponse__Unit()
    )


def test_client_sends_list_stores_on_existing_database_succeeds(spin_up_ahnlich_db):
    port = spin_up_ahnlich_db
    db_client = AhnlichDBClient(address="127.0.0.1", port=port)
    try:
        _ = db_client.create_store(**store_payload_no_predicates)
        response: db_response.ServerResult = db_client.list_stores()
    finally:
        db_client.cleanup()
    store_list: db_response.ServerResponse__StoreList = response.results[0].value
    store_info: db_response.StoreInfo = store_list.value[0]
    assert store_info.name == store_payload_no_predicates["store_name"]
    assert isinstance(response.results[0], db_response.Result__Ok)


def test_client_sends_create_stores_with_predicates_succeeds(module_scopped_ahnlich_db):
    port = module_scopped_ahnlich_db
    db_client = AhnlichDBClient(address="127.0.0.1", port=port)
    try:
        response: db_response.ServerResult = db_client.create_store(
            **store_payload_with_predicates
        )
    finally:
        db_client.cleanup()

    assert response.results[0] == db_response.Result__Ok(
        db_response.ServerResponse__Unit()
    )


def test_client_list_stores_finds_created_store_with_predicate(spin_up_ahnlich_db):
    port = spin_up_ahnlich_db
    db_client = AhnlichDBClient(address="127.0.0.1", port=port)
    try:
        _ = db_client.create_store(**store_payload_with_predicates)
        response: db_response.ServerResult = db_client.list_stores()
    finally:
        db_client.cleanup()
    assert isinstance(response.results[0], db_response.Result__Ok)

    store_lists: db_response.ServerResponse__StoreList = response.results[0].value
    assert len(store_lists.value) == 1
    store_info: db_response.StoreInfo = store_lists.value[0]
    queried_store_names = []
    for store_info in store_lists.value:
        queried_store_names.append(store_info.name)
    assert store_payload_with_predicates["store_name"] in queried_store_names


def test_client_set_in_store_succeeds(spin_up_ahnlich_db, store_key, store_value):
    port = spin_up_ahnlich_db
    db_client = AhnlichDBClient(address="127.0.0.1", port=port)
    store_key_2 = create_store_key(data=[5.0, 3.0, 4.0, 3.9, 4.9])

    # prepare data
    store_data = {
        "store_name": store_payload_no_predicates["store_name"],
        "inputs": [
            (store_key, store_value),
            (store_key_2, {"rank": db_query.MetadataValue__RawString("chunin")}),
        ],
    }
    # process data
    try:
        _ = db_client.create_store(**store_payload_no_predicates)
        response: db_response.ServerResult = db_client.set(**store_data)
    finally:
        db_client.cleanup()

    assert isinstance(response.results[0], db_response.Result__Ok)

    assert response.results[0] == db_response.Result__Ok(
        db_response.ServerResponse__Set(db_response.StoreUpsert(inserted=2, updated=0))
    )


def test_client_set_in_store_succeeds_with_binary(spin_up_ahnlich_db):
    port = spin_up_ahnlich_db
    db_client = AhnlichDBClient(address="127.0.0.1", port=port)
    store_key = create_store_key(data=[1.0, 4.0, 3.0, 3.9, 4.9])

    # prepare data
    store_data = {
        "store_name": store_payload_no_predicates["store_name"],
        "inputs": [
            (
                store_key,
                {"image": db_query.MetadataValue__Image(value=[2, 2, 3, 4, 5, 6, 7])},
            ),
        ],
    }
    # process data
    try:
        _ = db_client.create_store(**store_payload_no_predicates)
        response: db_response.ServerResult = db_client.set(**store_data)
    finally:
        db_client.cleanup()

    assert isinstance(response.results[0], db_response.Result__Ok)

    assert response.results[0] == db_response.Result__Ok(
        db_response.ServerResponse__Set(db_response.StoreUpsert(inserted=1, updated=0))
    )


def test_client_get_key_succeeds(module_scopped_ahnlich_db, store_key, store_value):
    port = module_scopped_ahnlich_db
    db_client = AhnlichDBClient(address="127.0.0.1", port=port)

    # prepare data
    get_key_data = {
        "store_name": store_payload_no_predicates["store_name"],
        "keys": [store_key],
    }
    store_key_2 = create_store_key(data=[5.0, 3.0, 4.0, 3.9, 4.9])
    # process data
    try:
        builder = db_client.pipeline()
        builder.create_store(**store_payload_no_predicates)
        builder.set(
            store_name=store_payload_no_predicates["store_name"],
            inputs=[
                (store_key, store_value),
                (store_key_2, {"job": db_query.MetadataValue__RawString("Assassin")}),
            ],
        )
        _ = builder.exec()
        response: db_response.ServerResult = db_client.get_key(**get_key_data)
    finally:
        db_client.cleanup()
    assert isinstance(response.results[0], db_response.Result__Ok)
    expected_result = [(store_key, store_value)]
    actual_response = response.results[0].value.value
    assert len(actual_response) == len(expected_result)
    assert actual_response[0][0].data == store_key.data


# def test_client_get_by_predicate_succeeds_with_no_index_in_store(spin_up_ahnlich_db):
#     port = spin_up_ahnlich_db
#     db_client = AhnlichDBClient(address="127.0.0.1", port=port)

#     # prepare data
#     get_predicate_data = {
#         "store_name": store_payload_no_predicates["store_name"],
#         "condition": db_query.PredicateCondition__Value(
#             db_query.Predicate__Equals(
#                 key="job", value=db_query.MetadataValue__RawString("sorcerer")
#             )
#         ),
#     }
#     # process data
#     try:
#         response: db_response.ServerResult = db_client.get_by_predicate(
#             **get_predicate_data
#         )
#     finally:
#         db_client.cleanup()
#     assert isinstance(response.results[0], db_response.Result__Ok)


def test_client_create_pred_index_succeeds(spin_up_ahnlich_db):
    port = spin_up_ahnlich_db
    db_client = AhnlichDBClient(address="127.0.0.1", port=port)

    create_pred_index_data = {
        "store_name": store_payload_no_predicates["store_name"],
        "predicates": ["job", "rank"],
    }
    try:
        _ = db_client.create_store(**store_payload_no_predicates)
        response: db_response.ServerResult = db_client.create_pred_index(
            **create_pred_index_data
        )
    finally:
        db_client.cleanup()
    assert response.results[0] == db_response.Result__Ok(
        db_response.ServerResponse__CreateIndex(2)
    )


def test_client_get_by_predicate_succeeds(spin_up_ahnlich_db, store_key, store_value):
    port = spin_up_ahnlich_db
    db_client = AhnlichDBClient(address="127.0.0.1", port=port)

    # prepare data
    get_predicate_data = {
        "store_name": store_payload_no_predicates["store_name"],
        "condition": db_query.PredicateCondition__Value(
            db_query.Predicate__Equals(
                key="job", value=db_query.MetadataValue__RawString(value="sorcerer")
            )
        ),
    }
    # process data
    try:

        builder = db_client.pipeline()
        builder.create_store(**store_payload_no_predicates)
        builder.set(
            store_name=store_payload_no_predicates["store_name"],
            inputs=[
                (store_key, store_value),
            ],
        )
        _ = builder.exec()

        response: db_response.ServerResult = db_client.get_by_predicate(
            **get_predicate_data
        )
    finally:
        db_client.cleanup()
    assert isinstance(response.results[0], db_response.Result__Ok)
    expected_result = [(store_key, store_value)]
    actual_response = response.results[0].value.value
    assert len(actual_response) == len(expected_result)
    assert actual_response[0][0].data == store_key.data


def assert_store_value(
    store_value_1: typing.Dict[str, db_query.MetadataValue],
    store_value_2: typing.Dict[str, db_query.MetadataValue],
):

    for key_1, key_2 in zip(store_value_1.keys(), store_value_2.keys()):
        assert key_1 == key_2
        assert store_value_1[key_1].value == store_value_2[key_2].value


def test_client_get_sim_n_succeeds(spin_up_ahnlich_db, store_key, store_value):
    port = spin_up_ahnlich_db
    db_client = AhnlichDBClient(address="127.0.0.1", port=port)

    # closest to 1.0,2.0,3.0,4.0,5.0
    search_input = create_store_key(data=[1.0, 2.0, 3.0, 3.9, 4.9])
    store_key_2 = create_store_key(data=[5.0, 3.0, 4.0, 3.9, 4.9])

    # prepare data
    get_sim_n_data = {
        "store_name": store_payload_no_predicates["store_name"],
        "closest_n": 1,
        "search_input": search_input,
        "algorithm": db_query.Algorithm__CosineSimilarity(),
    }

    # prepare data
    store_data = {
        "store_name": store_payload_no_predicates["store_name"],
        "inputs": [
            (store_key, store_value),
            (store_key_2, {"job": db_query.MetadataValue__RawString("assassin")}),
        ],
    }
    # process data
    try:
        builder = db_client.pipeline()
        builder.create_store(**store_payload_no_predicates)
        builder.set(**store_data)
        _ = builder.exec()
        response: db_response.ServerResult = db_client.get_sim_n(**get_sim_n_data)

    finally:
        db_client.cleanup()
    actual_results: db_response.ServerResponse__GetSimN = response.results[
        0
    ].value.value

    expected_results = [
        store_key,
        store_value,
        0.9999504,
    ]
    assert len(actual_results) == 1
    assert_store_value(actual_results[0][1], expected_results[1])
    assert str(expected_results[2]) in str(actual_results[0]).lower()


def test_client_drop_pred_index_succeeds(spin_up_ahnlich_db):
    port = spin_up_ahnlich_db
    db_client = AhnlichDBClient(address="127.0.0.1", port=port)

    create_pred_index_data = {
        "store_name": store_payload_no_predicates["store_name"],
        "predicates": ["to_drop"],
    }
    _ = db_client.create_store(**store_payload_no_predicates)
    response: db_response.ServerResult = db_client.create_pred_index(
        **create_pred_index_data
    )
    assert response.results[0] == db_response.Result__Ok(
        db_response.ServerResponse__CreateIndex(1)
    )

    drop_pred_index_data = {
        "store_name": store_payload_no_predicates["store_name"],
        "predicates": ["to_drop"],
        "error_if_not_exists": True,
    }

    try:
        response: db_response.ServerResult = db_client.drop_pred_index(
            **drop_pred_index_data
        )
    finally:
        db_client.cleanup()
    assert response.results[0] == db_response.Result__Ok(
        db_response.ServerResponse__Del(1)
    )


def test_client_delete_predicate_succeeds(spin_up_ahnlich_db, store_key, store_value):
    port = spin_up_ahnlich_db
    db_client = AhnlichDBClient(address="127.0.0.1", port=port)

    delete_predicate_data = {
        "store_name": store_payload_no_predicates["store_name"],
        "condition": db_query.PredicateCondition__Value(
            db_query.Predicate__Equals(
                key="rank", value=db_query.MetadataValue__RawString("chunin")
            )
        ),
    }
    store_key_2 = create_store_key(data=[5.0, 3.0, 4.0, 3.9, 4.9])
    # prepare data
    store_data = {
        "store_name": store_payload_no_predicates["store_name"],
        "inputs": [
            (store_key, store_value),
            (store_key_2, {"rank": db_query.MetadataValue__RawString("chunin")}),
        ],
    }

    try:
        builder = db_client.pipeline()

        builder.create_store(**store_payload_no_predicates)
        builder.create_pred_index(
            store_payload_no_predicates["store_name"], predicates=["rank"]
        )
        builder.set(**store_data)
        _ = builder.exec()

        response: db_response.ServerResult = db_client.delete_predicate(
            **delete_predicate_data
        )
    finally:
        db_client.cleanup()
    assert response.results[0] == db_response.Result__Ok(
        db_response.ServerResponse__Del(1)
    )


def test_client_delete_key_succeeds(spin_up_ahnlich_db, store_key, store_value):
    port = spin_up_ahnlich_db
    db_client = AhnlichDBClient(address="127.0.0.1", port=port)

    delete_key_data = {
        "store_name": store_payload_no_predicates["store_name"],
        "keys": [store_key],
    }
    store_data = {
        "store_name": store_payload_no_predicates["store_name"],
        "inputs": [(store_key, store_value)],
    }

    try:

        builder = db_client.pipeline()
        builder.create_store(**store_payload_no_predicates)
        builder.set(**store_data)
        _ = builder.exec()
        response: db_response.ServerResult = db_client.delete_key(**delete_key_data)
    finally:
        db_client.cleanup()
    assert response.results[0] == db_response.Result__Ok(
        db_response.ServerResponse__Del(1)
    )


def test_client_drop_store_succeeds(spin_up_ahnlich_db):
    port = spin_up_ahnlich_db
    db_client = AhnlichDBClient(address="127.0.0.1", port=port)

    drop_store_data = {
        "store_name": store_payload_no_predicates["store_name"],
        "error_if_not_exists": True,
    }

    try:
        builder = db_client.pipeline()
        builder.create_store(**store_payload_no_predicates)
        builder.exec()
        response: db_response.ServerResult = db_client.drop_store(**drop_store_data)
    finally:
        db_client.cleanup()
    assert response.results[0] == db_response.Result__Ok(
        db_response.ServerResponse__Del(1)
    )


def test_client_drop_store_fails_no_store(spin_up_ahnlich_db):
    port = spin_up_ahnlich_db
    db_client = AhnlichDBClient(address="127.0.0.1", port=port)
    store_name = store_payload_no_predicates["store_name"]
    drop_store_data = {
        "store_name": store_name,
        "error_if_not_exists": True,
    }

    try:
        response: db_response.ServerResult = db_client.drop_store(**drop_store_data)
    finally:
        db_client.cleanup()
    assert response.results[0] == db_response.Result__Err(
        value=f"Store {store_name} not found"
    )


def test_client_list_stores_reflects_dropped_store(
    module_scopped_ahnlich_db,
):
    port = module_scopped_ahnlich_db
    db_client = AhnlichDBClient(address="127.0.0.1", port=port)
    try:
        builder = db_client.pipeline()
        builder.create_store(**store_payload_no_predicates)
        builder.create_store(**store_payload_with_predicates)
        builder.drop_store(
            store_name=store_payload_no_predicates["store_name"],
            error_if_not_exists=True,
        )
        builder.exec()
        response: db_response.ServerResult = db_client.list_stores()
    finally:
        db_client.cleanup()
    store_list: db_response.ServerResponse__StoreList = response.results[0].value
    assert len(store_list.value) == 1
    store_info: db_response.StoreInfo = store_list.value[0]
    assert store_info.name == store_payload_with_predicates["store_name"]
    assert isinstance(response.results[0], db_response.Result__Ok)
