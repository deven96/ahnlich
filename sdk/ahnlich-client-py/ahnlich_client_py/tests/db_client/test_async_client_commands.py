import pytest

from ahnlich_client_py.clients import non_blocking
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


@pytest.mark.asyncio
async def test_client_sends_ping_to_db_success(module_scopped_ahnlich_db):
    port = module_scopped_ahnlich_db
    db_client = non_blocking.AhnlichDBClient(address="127.0.0.1", port=port)
    try:
        response: db_response.ServerResult = await db_client.ping()
        assert len(response.results) == 1
        assert response.results[0] == db_response.Result__Ok(
            db_response.ServerResponse__Pong()
        )

    except Exception as e:
        print(f"Exception: {e}")
        await db_client.cleanup()
        raise e
    finally:
        await db_client.cleanup()


@pytest.mark.asyncio
async def test_client_sends_list_clients_to_db_success(module_scopped_ahnlich_db):
    port = module_scopped_ahnlich_db
    db_client = non_blocking.AhnlichDBClient(address="127.0.0.1", port=port)
    try:
        response: db_response.ServerResult = await db_client.list_clients()
        assert len(response.results) == 1
    except Exception as e:
        print(f"Exception: {e}")
        await db_client.cleanup()
        raise e
    finally:
        await db_client.cleanup()


@pytest.mark.asyncio
async def test_client_sends_info_server_to_db_success(module_scopped_ahnlich_db):
    port = module_scopped_ahnlich_db

    db_client = non_blocking.AhnlichDBClient(address="127.0.0.1", port=port)

    try:
        response: db_response.ServerResult = await db_client.info_server()
        assert len(response.results) == 1
        info_server: db_response.ServerInfo = response.results[0].value
        assert info_server.value.version == db_client.message_protocol.version
        assert info_server.value.type == db_response.ServerType__Database()
    except Exception as e:
        print(f"Exception: {e}")
        await db_client.cleanup()
        raise e
    finally:
        await db_client.cleanup()


@pytest.mark.asyncio
async def test_client_sends_create_stores_succeeds(spin_up_ahnlich_db):
    port = spin_up_ahnlich_db

    db_client = non_blocking.AhnlichDBClient(address="127.0.0.1", port=port)
    try:
        response: db_response.ServerResult = await db_client.create_store(
            **store_payload_no_predicates
        )
    except Exception as e:
        print(f"Exception: {e}")
    finally:
        await db_client.cleanup()
    assert response.results[0] == db_response.Result__Ok(
        db_response.ServerResponse__Unit()
    )


@pytest.mark.asyncio
async def test_client_sends_list_stores_on_existing_database_succeeds(
    spin_up_ahnlich_db,
):
    port = spin_up_ahnlich_db
    db_client = non_blocking.AhnlichDBClient(address="127.0.0.1", port=port)
    try:
        _ = await db_client.create_store(**store_payload_no_predicates)
        response: db_response.ServerResult = await db_client.list_stores()
    except Exception as e:
        print(f"Exception: {e}")
    finally:
        await db_client.cleanup()
    store_list: db_response.ServerResponse__StoreList = response.results[0].value
    store_info: db_response.StoreInfo = store_list.value[0]
    assert store_info.name == store_payload_no_predicates["store_name"]
    assert isinstance(response.results[0], db_response.Result__Ok)


@pytest.mark.asyncio
async def test_client_list_stores_finds_created_store_with_predicate(
    spin_up_ahnlich_db,
):
    port = spin_up_ahnlich_db
    db_client = non_blocking.AhnlichDBClient(address="127.0.0.1", port=port)
    try:
        _ = await db_client.create_store(**store_payload_with_predicates)
        response: db_response.ServerResult = await db_client.list_stores()

        assert isinstance(response.results[0], db_response.Result__Ok)
        store_lists: db_response.ServerResponse__StoreList = response.results[0].value
        assert len(store_lists.value) == 1
        store_info: db_response.StoreInfo = store_lists.value[0]
        queried_store_names = []
        for store_info in store_lists.value:
            queried_store_names.append(store_info.name)
        assert store_payload_with_predicates["store_name"] in queried_store_names

    except Exception as e:
        print(f"Exception: {e}")
    finally:
        await db_client.cleanup()


@pytest.mark.asyncio
async def test_client_set_in_store_succeeds(spin_up_ahnlich_db, store_key, store_value):
    port = spin_up_ahnlich_db
    db_client = non_blocking.AhnlichDBClient(address="127.0.0.1", port=port)
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
        _ = await db_client.create_store(**store_payload_no_predicates)
        response: db_response.ServerResult = await db_client.set(**store_data)

        assert isinstance(response.results[0], db_response.Result__Ok)

        assert response.results[0] == db_response.Result__Ok(
            db_response.ServerResponse__Set(
                db_response.StoreUpsert(inserted=2, updated=0)
            )
        )

    except Exception as e:
        print(f"Exception: {e}")
    finally:
        await db_client.cleanup()


@pytest.mark.asyncio
async def test_client_set_in_store_succeeds_with_binary(spin_up_ahnlich_db):
    port = spin_up_ahnlich_db
    db_client = non_blocking.AhnlichDBClient(address="127.0.0.1", port=port)
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
        _ = await db_client.create_store(**store_payload_no_predicates)
        response: db_response.ServerResult = await db_client.set(**store_data)

        assert isinstance(response.results[0], db_response.Result__Ok)

        assert response.results[0] == db_response.Result__Ok(
            db_response.ServerResponse__Set(
                db_response.StoreUpsert(inserted=1, updated=0)
            )
        )

    except Exception as e:
        print(f"Exception: {e}")
    finally:
        await db_client.cleanup()


@pytest.mark.asyncio
async def test_client_get_key_succeeds(
    module_scopped_ahnlich_db, store_key, store_value
):
    port = module_scopped_ahnlich_db
    db_client = non_blocking.AhnlichDBClient(address="127.0.0.1", port=port)

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
        _ = await builder.exec()
        response: db_response.ServerResult = await db_client.get_key(**get_key_data)
        assert isinstance(response.results[0], db_response.Result__Ok)
        expected_result = [(store_key, store_value)]
        actual_response = response.results[0].value.value
        assert len(actual_response) == len(expected_result)
        assert actual_response[0][0].data == store_key.data

    except Exception as e:
        print(f"Exception: {e}")
    finally:
        await db_client.cleanup()
