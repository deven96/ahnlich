import grpclib
import pytest
from grpclib.client import Channel
from grpclib.exceptions import GRPCError

from ahnlich_client_py.grpc import keyval, metadata, predicates
from ahnlich_client_py.grpc.ai import pipeline, preprocess
from ahnlich_client_py.grpc.ai import query as ai_query
from ahnlich_client_py.grpc.ai import server as ai_server
from ahnlich_client_py.grpc.ai.models import AiModel
from ahnlich_client_py.grpc.services import ai_service

# Test data setup
ai_store_payload_no_predicates = {
    "store": "Diretnan Stores",
    "query_model": AiModel.ALL_MINI_LM_L6_V2,
    "index_model": AiModel.ALL_MINI_LM_L6_V2,
    "error_if_exists": True,
    "store_original": True,
}

ai_store_payload_with_predicates = {
    "store": "Diretnan Predication Stores",
    "query_model": AiModel.ALL_MINI_LM_L6_V2,
    "index_model": AiModel.ALL_MINI_LM_L6_V2,
    "predicates": ["special", "brand"],
    "error_if_exists": True,
    "store_original": True,
}


@pytest.mark.asyncio
async def test_aiproxy_client_sends_create_stores_succeeds(spin_up_ahnlich_ai):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_ai)
    client = ai_service.AiServiceStub(channel)
    try:
        request = ai_query.CreateStore(**ai_store_payload_no_predicates)
        response = await client.create_store(request)
        assert isinstance(response, ai_server.Unit)
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_create_store_fails_when_exists(spin_up_ahnlich_ai):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_ai)
    client = ai_service.AiServiceStub(channel)
    try:
        await client.create_store(
            ai_query.CreateStore(**ai_store_payload_no_predicates)
        )
        with pytest.raises(GRPCError) as exc_info:
            await client.create_store(
                ai_query.CreateStore(**ai_store_payload_no_predicates)
            )
        assert exc_info.value.status == grpclib.Status.ALREADY_EXISTS
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_ai_client_get_pred(spin_up_ahnlich_ai):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_ai)
    client = ai_service.AiServiceStub(channel)
    try:
        # Create store
        await client.create_store(
            ai_query.CreateStore(**ai_store_payload_with_predicates)
        )

        # Insert data
        entries = [
            keyval.AiStoreEntry(
                key=keyval.StoreInput(raw_string="Jordan One"),
                value=keyval.StoreValue(
                    value={"brand": metadata.MetadataValue(raw_string="Nike")}
                ),
            ),
            keyval.AiStoreEntry(
                key=keyval.StoreInput(raw_string="Yeezey"),
                value=keyval.StoreValue(
                    value={"brand": metadata.MetadataValue(raw_string="Adidas")}
                ),
            ),
        ]
        await client.set(
            ai_query.Set(
                store=ai_store_payload_with_predicates["store"],
                inputs=entries,
                preprocess_action=preprocess.PreprocessAction.NoPreprocessing,
            )
        )

        # Query by predicate
        condition = predicates.PredicateCondition(
            value=predicates.Predicate(
                equals=predicates.Equals(
                    key="brand", value=metadata.MetadataValue(raw_string="Nike")
                )
            )
        )
        response = await client.get_pred(
            ai_query.GetPred(
                store=ai_store_payload_with_predicates["store"], condition=condition
            )
        )

        assert len(response.entries) == 1
        print(response.entries)
        assert response.entries[0].key.raw_string == "Jordan One"
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_ai_client_create_pred_index(spin_up_ahnlich_ai):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_ai)
    client = ai_service.AiServiceStub(channel)
    try:
        # Create store
        await client.create_store(
            ai_query.CreateStore(**ai_store_payload_no_predicates)
        )

        # Create index
        response = await client.create_pred_index(
            ai_query.CreatePredIndex(
                store=ai_store_payload_no_predicates["store"],
                predicates=["super_sales"],
            )
        )

        assert response.created_indexes == 1
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_ai_client_drop_pred_index(spin_up_ahnlich_ai):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_ai)
    client = ai_service.AiServiceStub(channel)
    try:
        # Create store and index
        await client.create_store(
            ai_query.CreateStore(**ai_store_payload_no_predicates)
        )
        await client.create_pred_index(
            ai_query.CreatePredIndex(
                store=ai_store_payload_no_predicates["store"],
                predicates=["super_sales", "testing", "no mass"],
            )
        )

        # Drop existing index
        drop_response = await client.drop_pred_index(
            ai_query.DropPredIndex(
                store=ai_store_payload_no_predicates["store"],
                predicates=["testing"],
                error_if_not_exists=True,
            )
        )
        assert drop_response.deleted_count == 1

        # Attempt to drop non-existent index
        with pytest.raises(GRPCError) as exc_info:
            await client.drop_pred_index(
                ai_query.DropPredIndex(
                    store=ai_store_payload_no_predicates["store"],
                    predicates=["fake_predicate"],
                    error_if_not_exists=True,
                )
            )
        assert exc_info.value.status == grpclib.Status.NOT_FOUND
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_ai_client_del_key(spin_up_ahnlich_ai):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_ai)
    client = ai_service.AiServiceStub(channel)
    try:
        # Create store and insert data
        await client.create_store(
            ai_query.CreateStore(**ai_store_payload_with_predicates)
        )

        entries = [
            keyval.AiStoreEntry(
                key=keyval.StoreInput(raw_string="Jordan One"),
                value=keyval.StoreValue(
                    value={"brand": metadata.MetadataValue(raw_string="Nike")}
                ),
            ),
            keyval.AiStoreEntry(
                key=keyval.StoreInput(raw_string="Yeezey"),
                value=keyval.StoreValue(
                    value={"brand": metadata.MetadataValue(raw_string="Adidas")}
                ),
            ),
        ]
        await client.set(
            ai_query.Set(
                store=ai_store_payload_with_predicates["store"],
                inputs=entries,
                preprocess_action=preprocess.PreprocessAction.NoPreprocessing,
            )
        )

        # Delete key
        response = await client.del_key(
            ai_query.DelKey(
                store=ai_store_payload_with_predicates["store"],
                keys=[keyval.StoreInput(raw_string="Yeezey")],
            )
        )

        assert response.deleted_count == 1
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_ai_client_get_key(spin_up_ahnlich_ai):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_ai)
    client = ai_service.AiServiceStub(channel)
    try:
        # Create store and insert data
        await client.create_store(
            ai_query.CreateStore(**ai_store_payload_with_predicates)
        )

        entries = [
            keyval.AiStoreEntry(
                key=keyval.StoreInput(raw_string="Jordan One"),
                value=keyval.StoreValue(value={}),
            )
        ]
        await client.set(
            ai_query.Set(
                store=ai_store_payload_with_predicates["store"],
                inputs=entries,
                preprocess_action=preprocess.PreprocessAction.NoPreprocessing,
            )
        )

        # Get key
        response = await client.get_key(
            ai_query.GetKey(
                store=ai_store_payload_with_predicates["store"],
                keys=[keyval.StoreInput(raw_string="Jordan One")],
            )
        )

        assert len(response.entries) == 1
        assert response.entries[0].key.raw_string == "Jordan One"
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_ai_client_drop_store_succeeds(spin_up_ahnlich_ai):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_ai)
    client = ai_service.AiServiceStub(channel)
    try:
        # Create stores
        await client.create_store(
            ai_query.CreateStore(**ai_store_payload_no_predicates)
        )
        await client.create_store(
            ai_query.CreateStore(**ai_store_payload_with_predicates)
        )

        # Drop one store
        response = await client.drop_store(
            ai_query.DropStore(
                store=ai_store_payload_with_predicates["store"],
                error_if_not_exists=True,
            )
        )

        assert response.deleted_count == 1
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_ai_client_purge_stores_succeeds(spin_up_ahnlich_ai):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_ai)
    client = ai_service.AiServiceStub(channel)
    try:
        # Create stores
        await client.create_store(
            ai_query.CreateStore(**ai_store_payload_no_predicates)
        )
        await client.create_store(
            ai_query.CreateStore(**ai_store_payload_with_predicates)
        )

        # Purge all stores
        response = await client.purge_stores(ai_query.PurgeStores())

        assert response.deleted_count == 2
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_ai_client_list_clients_succeeds(spin_up_ahnlich_ai):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_ai)
    client = ai_service.AiServiceStub(channel)
    try:
        response = await client.list_clients(ai_query.ListClients())
        assert len(response.clients) >= 1  # At least our connection
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_ai_pipeline_multiple_operations(spin_up_ahnlich_ai):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_ai)
    client = ai_service.AiServiceStub(channel)
    try:
        # Create pipeline request
        pipeline_request = pipeline.AiRequestPipeline(
            queries=[
                pipeline.AiQuery(
                    create_store=ai_query.CreateStore(
                        **ai_store_payload_with_predicates
                    )
                ),
                pipeline.AiQuery(
                    set=ai_query.Set(
                        store=ai_store_payload_with_predicates["store"],
                        inputs=[
                            keyval.AiStoreEntry(
                                key=keyval.StoreInput(raw_string="Product1"),
                                value=keyval.StoreValue(
                                    value={
                                        "category": metadata.MetadataValue(
                                            raw_string="Electronics"
                                        )
                                    }
                                ),
                            )
                        ],
                        preprocess_action=preprocess.PreprocessAction.NoPreprocessing,
                    )
                ),
                pipeline.AiQuery(
                    create_pred_index=ai_query.CreatePredIndex(
                        store=ai_store_payload_with_predicates["store"],
                        predicates=["category"],
                    )
                ),
            ]
        )

        response = await client.pipeline(pipeline_request)
        assert len(response.responses) == 3
        assert isinstance(response.responses[0].unit, ai_server.Unit)
        assert isinstance(response.responses[1].set, ai_server.Set)
        assert isinstance(response.responses[2].create_index, ai_server.CreateIndex)
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_ai_pipeline_with_error(spin_up_ahnlich_ai):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_ai)
    client = ai_service.AiServiceStub(channel)
    try:
        # Create pipeline with invalid operation (store doesn't exist)
        pipeline_request = pipeline.AiRequestPipeline(
            queries=[
                pipeline.AiQuery(
                    set=ai_query.Set(
                        store="nonexistent_store",
                        inputs=[
                            keyval.AiStoreEntry(
                                key=keyval.StoreInput(raw_string="Product1"),
                                value=keyval.StoreValue(
                                    value={
                                        "category": metadata.MetadataValue(
                                            raw_string="Electronics"
                                        )
                                    }
                                ),
                            )
                        ],
                    )
                )
            ]
        )

        response = await client.pipeline(pipeline_request)
        assert len(response.responses) == 1
        assert response.responses[0].error is not None
        assert "not found" in response.responses[0].error.message.lower()
    finally:
        channel.close()
