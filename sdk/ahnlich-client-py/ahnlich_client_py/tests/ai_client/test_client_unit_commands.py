from ahnlich_client_py import clients
from ahnlich_client_py.internals import ai_response


def test_client_sends_ping_to_aiproxy_success(module_scopped_ahnlich_ai):
    port = module_scopped_ahnlich_ai
    ai_client = clients.AhnlichAIClient(address="127.0.0.1", port=port)
    try:
        response: ai_response.AIServerResult = ai_client.ping()
        assert len(response.results) == 1
        assert response.results[0] == ai_response.Result__Ok(
            ai_response.AIServerResponse__Pong()
        )
    except Exception as e:
        print(f"Exception: {e}")
    finally:
        ai_client.cleanup()


def test_client_sends_info_server_to_aiproxy_success(module_scopped_ahnlich_ai):
    port = module_scopped_ahnlich_ai
    ai_client = clients.AhnlichAIClient(address="127.0.0.1", port=port)
    try:
        response: ai_response.AIServerResult = ai_client.info_server()
        assert len(response.results) == 1

        info_server: ai_response.ServerInfo = response.results[0].value
        assert (
            info_server.value.version.major == ai_client.message_protocol.version.major
        )
        assert info_server.value.type == ai_response.ServerType__AI()

    except Exception as e:
        print(f"Exception: {e}")
    finally:
        ai_client.cleanup()


def test_client_sends_list_stores_to_fresh_aiproxy_succeeds(module_scopped_ahnlich_ai):
    port = module_scopped_ahnlich_ai
    ai_client = clients.AhnlichAIClient(address="127.0.0.1", port=port)
    try:
        response: ai_response.AIServerResult = ai_client.list_stores()
        assert response.results[0] == ai_response.Result__Ok(
            ai_response.AIServerResponse__StoreList([])
        )
    except Exception as e:
        print(f"Exception: {e}")
    finally:
        ai_client.cleanup()


def test_client_works_using_protocol_in_context(module_scopped_ahnlich_ai):
    port = module_scopped_ahnlich_ai
    with clients.AhnlichAIClient(address="127.0.0.1", port=port) as ai_client:
        response: ai_response.AIServerResult = ai_client.list_stores()
    assert response.results[0] == ai_response.Result__Ok(
        ai_response.AIServerResponse__StoreList([])
    )
