import pytest

from ahnlich_client_py.clients import non_blocking
from ahnlich_client_py.internals import ai_query, ai_response

ai_store_payload_no_predicates = {
    "store_name": "Diretnan Stores",
    "query_model": ai_query.AIModel__AllMiniLML6V2(),
    "index_model": ai_query.AIModel__AllMiniLML6V2(),
}

ai_store_payload_with_predicates = {
    "store_name": "Diretnan Predication Stores",
    "query_model": ai_query.AIModel__AllMiniLML6V2(),
    "index_model": ai_query.AIModel__AllMiniLML6V2(),
    "predicates": ["special", "brand"],
}


@pytest.mark.asyncio
async def test_client_sends_ping_to_aiproxy_success(module_scopped_ahnlich_ai):
    port = module_scopped_ahnlich_ai
    ai_client = non_blocking.AhnlichAIClient(address="127.0.0.1", port=port)
    try:
        response: ai_response.AIServerResult = await ai_client.ping()
        assert len(response.results) == 1
        assert response.results[0] == ai_response.Result__Ok(
            ai_response.AIServerResponse__Pong()
        )
    except Exception as e:
        print(f"Exception: {e}")
    finally:
        await ai_client.cleanup()


@pytest.mark.asyncio
async def test_client_sends_info_server_to_aiproxy_success(module_scopped_ahnlich_ai):
    port = module_scopped_ahnlich_ai
    ai_client = non_blocking.AhnlichAIClient(address="127.0.0.1", port=port)
    try:
        response: ai_response.AIServerResult = await ai_client.info_server()
        assert len(response.results) == 1

        info_server: ai_response.ServerInfo = response.results[0].value
        assert (
            info_server.value.version.major == ai_client.message_protocol.version.major
        )
        assert info_server.value.type == ai_response.ServerType__AI()

    except Exception as e:
        print(f"Exception: {e}")
        await ai_client.cleanup()
        raise e
    finally:
        await ai_client.cleanup()


@pytest.mark.asyncio
async def test_client_sends_list_stores_to_fresh_aiproxy_succeeds(
    module_scopped_ahnlich_ai,
):
    port = module_scopped_ahnlich_ai
    ai_client = non_blocking.AhnlichAIClient(address="127.0.0.1", port=port)
    try:
        response: ai_response.AIServerResult = await ai_client.list_stores()
        assert response.results[0] == ai_response.Result__Ok(
            ai_response.AIServerResponse__StoreList([])
        )
    except Exception as e:
        print(f"Exception: {e}")
        await ai_client.cleanup()
        raise e
    finally:
        await ai_client.cleanup()


@pytest.mark.asyncio
async def test_aiproxy_client_sends_create_stores_succeeds(spin_up_ahnlich_ai):
    port = spin_up_ahnlich_ai

    ai_client = non_blocking.AhnlichAIClient(
        address="127.0.0.1", port=port, connect_timeout_sec=45
    )
    try:
        response: ai_response.AIServerResult = await ai_client.create_store(
            **ai_store_payload_no_predicates
        )
        assert response.results[0] == ai_response.Result__Ok(
            ai_response.AIServerResponse__Unit()
        )
    except Exception as e:
        await ai_client.cleanup()
        print(f"Exception: {e}")
        raise e
    finally:
        await ai_client.cleanup()


@pytest.mark.asyncio
async def test_ai_client_get_pred(spin_up_ahnlich_ai):
    port = spin_up_ahnlich_ai

    ai_client = non_blocking.AhnlichAIClient(address="127.0.0.1", port=port)
    store_inputs = [
        (
            ai_query.StoreInput__RawString("Jordan One"),
            {"brand": ai_query.MetadataValue__RawString("Nike")},
        ),
        (
            ai_query.StoreInput__RawString("Yeezey"),
            {"brand": ai_query.MetadataValue__RawString("Adidas")},
        ),
    ]
    builder = ai_client.pipeline()
    builder.create_store(**ai_store_payload_with_predicates)
    builder.set(
        store_name=ai_store_payload_with_predicates["store_name"],
        inputs=store_inputs,
        preprocess_action=ai_query.PreprocessAction__NoPreprocessing(),
    )
    expected = ai_response.AIServerResult(
        results=[
            ai_response.Result__Ok(
                value=ai_response.AIServerResponse__Get(
                    value=[
                        (
                            ai_query.StoreInput__RawString(value="Jordan One"),
                            {"brand": ai_query.MetadataValue__RawString(value="Nike")},
                        )
                    ]
                )
            )
        ]
    )

    try:
        await builder.exec()
        response = await ai_client.get_pred(
            ai_store_payload_with_predicates["store_name"],
            ai_query.PredicateCondition__Value(
                value=ai_query.Predicate__Equals(
                    key="brand", value=ai_query.MetadataValue__RawString("Nike")
                )
            ),
        )
        assert str(expected) == str(response)

    except Exception as e:
        print(f"Exception: {e}")
        await ai_client.cleanup()
        raise e
    finally:
        await ai_client.cleanup()


@pytest.mark.asyncio
async def test_ai_client_create_pred_index(spin_up_ahnlich_ai):
    port = spin_up_ahnlich_ai

    ai_client = non_blocking.AhnlichAIClient(address="127.0.0.1", port=port)

    try:
        builder = ai_client.pipeline()
        builder.create_store(**ai_store_payload_no_predicates)
        builder.list_stores()
        response = await builder.exec()
        response = await ai_client.create_pred_index(
            ai_store_payload_no_predicates["store_name"],
            predicates=["super_sales"],
        )
        expected = ai_response.AIServerResult(
            results=[
                ai_response.Result__Ok(ai_response.AIServerResponse__CreateIndex(1))
            ]
        )
        assert str(response) == str(expected)
    except Exception as e:
        print(f"Exception: {e}")
        await ai_client.cleanup()
        raise e
    finally:
        await ai_client.cleanup()
