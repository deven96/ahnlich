from ahnlich_client_py.clients import AhnlichAIClient
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


def test_aiproxy_client_sends_create_stores_succeeds(spin_up_ahnlich_ai):
    port = spin_up_ahnlich_ai

    ai_client = AhnlichAIClient(address="127.0.0.1", port=port, connect_timeout_sec=45)
    try:
        response: ai_response.AIServerResult = ai_client.create_store(
            **ai_store_payload_no_predicates
        )
        assert response.results[0] == ai_response.Result__Ok(
            ai_response.AIServerResponse__Unit()
        )
    except Exception as e:
        ai_client.cleanup()
        print(f"Exception: {e}")
        raise e
    finally:
        ai_client.cleanup()


def test_ai_client_get_pred(spin_up_ahnlich_ai):
    port = spin_up_ahnlich_ai

    ai_client = AhnlichAIClient(address="127.0.0.1", port=port)
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
        preprocess_action=ai_query.PreprocessAction__RawString(
            ai_query.StringAction__ErrorIfTokensExceed()
        ),
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
        builder.exec()
        response = ai_client.get_pred(
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
        ai_client.cleanup()
        raise e
    finally:
        ai_client.cleanup()


# TODO: once model is loaded into proxy, this can be done properly
# def test_ai_client_get_sim_n(spin_up_ahnlich_ai):
#     port = spin_up_ahnlich_ai


def test_ai_client_create_pred_index(spin_up_ahnlich_ai):
    port = spin_up_ahnlich_ai

    ai_client = AhnlichAIClient(address="127.0.0.1", port=port)

    try:
        builder = ai_client.pipeline()
        builder.create_store(**ai_store_payload_no_predicates)
        builder.list_stores()
        response = builder.exec()
        response = ai_client.create_pred_index(
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
        ai_client.cleanup()
        raise e
    finally:
        ai_client.cleanup()


def test_ai_client_drop_pred_index(spin_up_ahnlich_ai):
    port = spin_up_ahnlich_ai

    ai_client = AhnlichAIClient(address="127.0.0.1", port=port)

    try:
        builder = ai_client.pipeline()
        builder.create_store(**ai_store_payload_no_predicates)
        builder.list_stores()
        response = builder.exec()
        response = ai_client.create_pred_index(
            ai_store_payload_no_predicates["store_name"],
            predicates=["super_sales", "testing", "no mass"],
        )
        expected = ai_response.AIServerResult(
            results=[
                ai_response.Result__Ok(ai_response.AIServerResponse__CreateIndex(3))
            ]
        )
        assert str(response) == str(expected)

        builder = ai_client.pipeline()

        builder.drop_pred_index(
            ai_store_payload_no_predicates["store_name"],
            ["testing"],
            error_if_not_exists=True,
        )
        builder.drop_pred_index(
            ai_store_payload_no_predicates["store_name"],
            ["fake_predicate"],
            error_if_not_exists=True,
        )
        response_with_err = builder.exec()

        expected = ai_response.AIServerResult(
            results=[
                ai_response.Result__Ok(ai_response.AIServerResponse__Del(1)),
                ai_response.Result__Err(
                    value="db error Predicate fake_predicate not found in store, attempt CREATEPREDINDEX with predicate"
                ),
            ]
        )
        assert str(response_with_err) == str(expected)
    except Exception as e:
        print(f"Exception: {e}")
        ai_client.cleanup()
        raise e
    finally:
        ai_client.cleanup()


def test_ai_client_del_key(spin_up_ahnlich_ai):
    port = spin_up_ahnlich_ai

    ai_client = AhnlichAIClient(address="127.0.0.1", port=port)
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
        preprocess_action=ai_query.PreprocessAction__RawString(
            ai_query.StringAction__ErrorIfTokensExceed()
        ),
    )
    expected = ai_response.AIServerResult(
        results=[
            ai_response.Result__Ok(value=ai_response.AIServerResponse__Del(1)),
        ]
    )

    try:
        builder.exec()
        response = ai_client.del_key(
            ai_store_payload_with_predicates["store_name"],
            key=ai_query.StoreInput__RawString("Yeezey"),
        )

        assert str(expected) == str(response)
    except Exception as e:
        print(f"Exception: {e}")
        ai_client.cleanup()
        raise e
    finally:
        ai_client.cleanup()


def test_ai_client_drop_store_succeeds(spin_up_ahnlich_ai):
    port = spin_up_ahnlich_ai

    ai_client = AhnlichAIClient(address="127.0.0.1", port=port)
    expected = ai_response.AIServerResult(
        results=[
            ai_response.Result__Ok(ai_response.AIServerResponse__Del(1)),
        ]
    )

    try:
        builder = ai_client.pipeline()
        builder.create_store(**ai_store_payload_no_predicates)
        builder.create_store(**ai_store_payload_with_predicates)
        builder.list_stores()
        _ = builder.exec()
        response = ai_client.drop_store(
            store_name=ai_store_payload_with_predicates["store_name"],
            error_if_not_exists=True,
        )
        assert str(response) == str(expected)
    except Exception as e:
        print(f"Exception: {e}")
        ai_client.cleanup()
        raise e
    finally:
        ai_client.cleanup()


def test_ai_client_purge_stores_succeeds(spin_up_ahnlich_ai):
    port = spin_up_ahnlich_ai

    ai_client = AhnlichAIClient(address="127.0.0.1", port=port)
    expected = ai_response.AIServerResult(
        results=[
            ai_response.Result__Ok(ai_response.AIServerResponse__Del(2)),
        ]
    )

    try:
        builder = ai_client.pipeline()
        builder.create_store(**ai_store_payload_no_predicates)
        builder.create_store(**ai_store_payload_with_predicates)
        builder.list_stores()
        _ = builder.exec()

        response = ai_client.purge_stores()
        assert str(response) == str(expected)
    except Exception as e:
        print(f"Exception: {e}")
        ai_client.cleanup()
        raise e
    finally:
        ai_client.cleanup()
