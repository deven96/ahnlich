import grpclib
import pytest
from grpclib.client import Channel
from grpclib.exceptions import GRPCError

from ahnlich_client_py.grpc import keyval, metadata, predicates
from ahnlich_client_py.grpc.algorithm.algorithms import Algorithm
from ahnlich_client_py.grpc.algorithm.nonlinear import (
    HnswConfig,
    KdTreeConfig,
    NonLinearAlgorithm,
    NonLinearIndex,
)
from ahnlich_client_py.grpc.db import pipeline
from ahnlich_client_py.grpc.db import query as db_query
from ahnlich_client_py.grpc.db import server as db_server
from ahnlich_client_py.grpc.services import db_service

# Test data setup
store_payload_no_predicates = {
    "store": "Diretnan Station",
    "dimension": 5,
    "error_if_exists": True,
}

store_payload_with_predicates = {
    "store": "Diretnan Predication",
    "dimension": 5,
    "error_if_exists": True,
    "create_predicates": ["is_tyrannical", "rank"],
}


@pytest.mark.asyncio
async def test_client_sends_create_stores_succeeds(spin_up_ahnlich_db):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_db)
    client = db_service.DbServiceStub(channel)
    try:
        request = db_query.CreateStore(**store_payload_no_predicates)
        response = await client.create_store(request)
        assert isinstance(response, db_server.Unit)
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_client_sends_list_stores_on_existing_database_succeeds(
    spin_up_ahnlich_db,
):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_db)
    client = db_service.DbServiceStub(channel)
    try:
        # Create store first
        create_request = db_query.CreateStore(**store_payload_no_predicates)
        await client.create_store(create_request)

        # List stores
        list_request = db_query.ListStores()
        response = await client.list_stores(list_request)
        assert len(response.stores) == 1
        assert response.stores[0].name == store_payload_no_predicates["store"]
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_client_sends_create_stores_with_predicates_succeeds(spin_up_ahnlich_db):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_db)
    client = db_service.DbServiceStub(channel)
    try:
        request = db_query.CreateStore(**store_payload_with_predicates)
        response = await client.create_store(request)
        assert isinstance(response, db_server.Unit)
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_client_list_stores_finds_created_store_with_predicate(
    spin_up_ahnlich_db,
):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_db)
    client = db_service.DbServiceStub(channel)
    try:
        # Create store with predicates
        create_request = db_query.CreateStore(**store_payload_with_predicates)
        await client.create_store(create_request)

        # List stores
        list_request = db_query.ListStores()
        response = await client.list_stores(list_request)
        assert len(response.stores) == 1
        assert response.stores[0].name == store_payload_with_predicates["store"]
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_client_set_in_store_succeeds(spin_up_ahnlich_db):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_db)
    client = db_service.DbServiceStub(channel)
    try:
        # Create store
        create_request = db_query.CreateStore(**store_payload_no_predicates)
        await client.create_store(create_request)

        # Prepare set request
        store_key = keyval.StoreKey(key=[1.0, 2.0, 3.0, 4.0, 5.0])
        store_key_2 = keyval.StoreKey(key=[5.0, 3.0, 4.0, 3.9, 4.9])
        entries = [
            keyval.DbStoreEntry(
                key=store_key,
                value=keyval.StoreValue(
                    value={"job": metadata.MetadataValue(raw_string="sorcerer")}
                ),
            ),
            keyval.DbStoreEntry(
                key=store_key_2,
                value=keyval.StoreValue(
                    value={"rank": metadata.MetadataValue(raw_string="chunin")}
                ),
            ),
        ]
        set_request = db_query.Set(
            store=store_payload_no_predicates["store"], inputs=entries
        )

        # Execute set
        response = await client.set(set_request)
        assert response.upsert.inserted == 2
        assert response.upsert.updated == 0
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_client_set_in_store_succeeds_with_binary(spin_up_ahnlich_db):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_db)
    client = db_service.DbServiceStub(channel)
    try:
        # Create store
        create_request = db_query.CreateStore(**store_payload_no_predicates)
        await client.create_store(create_request)

        # Prepare binary data
        store_key = keyval.StoreKey(key=[1.0, 4.0, 3.0, 3.9, 4.9])
        entries = [
            keyval.DbStoreEntry(
                key=store_key,
                value=keyval.StoreValue(
                    value={
                        "image": metadata.MetadataValue(
                            image=bytes([2, 2, 3, 4, 5, 6, 7])
                        )
                    }
                ),
            )
        ]
        set_request = db_query.Set(
            store=store_payload_no_predicates["store"], inputs=entries
        )

        # Execute set
        response = await client.set(set_request)
        assert response.upsert.inserted == 1
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_client_get_key_succeeds(spin_up_ahnlich_db):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_db)
    client = db_service.DbServiceStub(channel)
    try:
        # Create store
        create_request = db_query.CreateStore(**store_payload_no_predicates)
        await client.create_store(create_request)

        # Insert data
        store_key = keyval.StoreKey(key=[1.0, 2.0, 3.0, 4.0, 5.0])
        store_key_2 = keyval.StoreKey(key=[5.0, 3.0, 4.0, 3.9, 4.9])
        entries = [
            keyval.DbStoreEntry(
                key=store_key,
                value=keyval.StoreValue(
                    value={"job": metadata.MetadataValue(raw_string="sorcerer")}
                ),
            ),
            keyval.DbStoreEntry(
                key=store_key_2,
                value=keyval.StoreValue(
                    value={"job": metadata.MetadataValue(raw_string="Assassin")}
                ),
            ),
        ]
        await client.set(
            db_query.Set(store=store_payload_no_predicates["store"], inputs=entries)
        )

        # Get key
        get_request = db_query.GetKey(
            store=store_payload_no_predicates["store"], keys=[store_key]
        )
        response = await client.get_key(get_request)
        assert len(response.entries) == 1
        assert response.entries[0].key == store_key
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_client_create_pred_index_succeeds(spin_up_ahnlich_db):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_db)
    client = db_service.DbServiceStub(channel)
    try:
        # Create store first
        create_request = db_query.CreateStore(**store_payload_no_predicates)
        await client.create_store(create_request)

        # Create index
        index_request = db_query.CreatePredIndex(
            store=store_payload_no_predicates["store"], predicates=["job", "rank"]
        )
        response = await client.create_pred_index(index_request)
        assert response.created_indexes == 2
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_client_get_by_predicate_succeeds(spin_up_ahnlich_db):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_db)
    client = db_service.DbServiceStub(channel)
    try:
        # Setup store with data
        create_request = db_query.CreateStore(**store_payload_no_predicates)
        await client.create_store(create_request)

        store_key = keyval.StoreKey(key=[1.0, 2.0, 3.0, 4.0, 5.0])
        entries = [
            keyval.DbStoreEntry(
                key=store_key,
                value=keyval.StoreValue(
                    value={"job": metadata.MetadataValue(raw_string="sorcerer")}
                ),
            )
        ]
        await client.set(
            db_query.Set(store=store_payload_no_predicates["store"], inputs=entries)
        )

        # Create predicate index
        await client.create_pred_index(
            db_query.CreatePredIndex(
                store=store_payload_no_predicates["store"], predicates=["job"]
            )
        )

        # Query by predicate
        condition = predicates.PredicateCondition(
            value=predicates.Predicate(
                equals=predicates.Equals(
                    key="job", value=metadata.MetadataValue(raw_string="sorcerer")
                )
            )
        )
        pred_request = db_query.GetPred(
            store=store_payload_no_predicates["store"], condition=condition
        )
        response = await client.get_pred(pred_request)
        assert len(response.entries) == 1
        assert response.entries[0].key == store_key
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_client_get_sim_n_succeeds(spin_up_ahnlich_db):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_db)
    client = db_service.DbServiceStub(channel)
    try:
        # Setup store with data
        create_request = db_query.CreateStore(**store_payload_no_predicates)
        await client.create_store(create_request)

        search_input = keyval.StoreKey(key=[1.0, 2.0, 3.0, 3.9, 4.9])
        store_key = keyval.StoreKey(key=[1.0, 2.0, 3.0, 4.0, 5.0])
        store_key_2 = keyval.StoreKey(key=[5.0, 3.0, 4.0, 3.9, 4.9])

        entries = [
            keyval.DbStoreEntry(
                key=store_key,
                value=keyval.StoreValue(
                    value={"job": metadata.MetadataValue(raw_string="sorcerer")}
                ),
            ),
            keyval.DbStoreEntry(
                key=store_key_2,
                value=keyval.StoreValue(
                    value={"job": metadata.MetadataValue(raw_string="assassin")}
                ),
            ),
        ]
        await client.set(
            db_query.Set(store=store_payload_no_predicates["store"], inputs=entries)
        )

        # Similarity search
        sim_request = db_query.GetSimN(
            store=store_payload_no_predicates["store"],
            search_input=search_input,
            closest_n=1,
            algorithm=Algorithm.CosineSimilarity,
        )
        response = await client.get_sim_n(sim_request)
        assert len(response.entries) == 1
        assert abs(response.entries[0].similarity.value - 0.999) < 0.001
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_client_drop_pred_index_succeeds(spin_up_ahnlich_db):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_db)
    client = db_service.DbServiceStub(channel)
    try:
        # Setup store and index
        create_request = db_query.CreateStore(**store_payload_no_predicates)
        await client.create_store(create_request)

        await client.create_pred_index(
            db_query.CreatePredIndex(
                store=store_payload_no_predicates["store"], predicates=["to_drop"]
            )
        )

        # Drop index
        drop_request = db_query.DropPredIndex(
            store=store_payload_no_predicates["store"],
            predicates=["to_drop"],
            error_if_not_exists=True,
        )
        response = await client.drop_pred_index(drop_request)
        assert response.deleted_count == 1
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_client_delete_predicate_succeeds(spin_up_ahnlich_db):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_db)
    client = db_service.DbServiceStub(channel)
    try:
        # Setup store with data and index
        create_request = db_query.CreateStore(**store_payload_no_predicates)
        await client.create_store(create_request)

        await client.create_pred_index(
            db_query.CreatePredIndex(
                store=store_payload_no_predicates["store"], predicates=["rank"]
            )
        )

        store_key = keyval.StoreKey(key=[1.0, 2.0, 3.0, 4.0, 5.0])
        store_key_2 = keyval.StoreKey(key=[5.0, 3.0, 4.0, 3.9, 4.9])
        entries = [
            keyval.DbStoreEntry(
                key=store_key,
                value=keyval.StoreValue(
                    value={"job": metadata.MetadataValue(raw_string="sorcerer")}
                ),
            ),
            keyval.DbStoreEntry(
                key=store_key_2,
                value=keyval.StoreValue(
                    value={"rank": metadata.MetadataValue(raw_string="chunin")}
                ),
            ),
        ]
        await client.set(
            db_query.Set(store=store_payload_no_predicates["store"], inputs=entries)
        )

        # Delete by predicate
        condition = predicates.PredicateCondition(
            value=predicates.Predicate(
                equals=predicates.Equals(
                    key="rank", value=metadata.MetadataValue(raw_string="chunin")
                )
            )
        )
        del_request = db_query.DelPred(
            store=store_payload_no_predicates["store"], condition=condition
        )
        response = await client.del_pred(del_request)
        assert response.deleted_count == 1
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_client_delete_key_succeeds(spin_up_ahnlich_db):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_db)
    client = db_service.DbServiceStub(channel)
    try:
        # Create store
        create_request = db_query.CreateStore(**store_payload_no_predicates)
        await client.create_store(create_request)

        # Insert data
        store_key = keyval.StoreKey(key=[1.0, 2.0, 3.0, 4.0, 5.0])
        entries = [
            keyval.DbStoreEntry(
                key=store_key,
                value=keyval.StoreValue(
                    value={"job": metadata.MetadataValue(raw_string="sorcerer")}
                ),
            )
        ]
        await client.set(
            db_query.Set(store=store_payload_no_predicates["store"], inputs=entries)
        )

        # Delete by key
        del_request = db_query.DelKey(
            store=store_payload_no_predicates["store"], keys=[store_key]
        )
        response = await client.del_key(del_request)
        assert response.deleted_count == 1
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_client_drop_store_succeeds(spin_up_ahnlich_db):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_db)
    client = db_service.DbServiceStub(channel)
    try:
        # Create store first
        create_request = db_query.CreateStore(**store_payload_no_predicates)
        await client.create_store(create_request)

        # Drop store
        drop_request = db_query.DropStore(
            store=store_payload_no_predicates["store"], error_if_not_exists=True
        )
        response = await client.drop_store(drop_request)
        assert response.deleted_count == 1
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_client_drop_store_fails_no_store(spin_up_ahnlich_db):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_db)
    client = db_service.DbServiceStub(channel)
    try:
        store_name = store_payload_no_predicates["store"]
        drop_request = db_query.DropStore(store=store_name, error_if_not_exists=True)

        with pytest.raises(GRPCError) as exc_info:
            await client.drop_store(drop_request)
        assert exc_info.value.status == grpclib.Status.NOT_FOUND
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_client_list_stores_reflects_dropped_store(spin_up_ahnlich_db):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_db)
    client = db_service.DbServiceStub(channel)
    try:
        # Create two stores
        await client.create_store(db_query.CreateStore(**store_payload_no_predicates))
        await client.create_store(db_query.CreateStore(**store_payload_with_predicates))

        # Drop one store
        await client.drop_store(
            db_query.DropStore(
                store=store_payload_no_predicates["store"], error_if_not_exists=True
            )
        )

        # Verify only one remains
        response = await client.list_stores(db_query.ListStores())
        assert len(response.stores) == 1
        assert response.stores[0].name == store_payload_with_predicates["store"]
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_client_create_and_drop_kdtree_non_linear_index(spin_up_ahnlich_db):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_db)
    client = db_service.DbServiceStub(channel)
    try:
        # Create store
        create_request = db_query.CreateStore(**store_payload_no_predicates)
        await client.create_store(create_request)

        # Create KDTree index
        create_index_request = db_query.CreateNonLinearAlgorithmIndex(
            store=store_payload_no_predicates["store"],
            non_linear_indices=[NonLinearIndex(kdtree=KdTreeConfig())],
        )
        response = await client.create_non_linear_algorithm_index(create_index_request)
        assert response.created_indexes == 1

        # Drop KDTree index
        drop_index_request = db_query.DropNonLinearAlgorithmIndex(
            store=store_payload_no_predicates["store"],
            non_linear_indices=[NonLinearAlgorithm.KDTree],
            error_if_not_exists=True,
        )
        response = await client.drop_non_linear_algorithm_index(drop_index_request)
        assert response.deleted_count == 1
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_client_create_and_drop_hnsw_non_linear_index(spin_up_ahnlich_db):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_db)
    client = db_service.DbServiceStub(channel)
    try:
        # Create store
        create_request = db_query.CreateStore(**store_payload_no_predicates)
        await client.create_store(create_request)

        # Create HNSW index with default config
        create_index_request = db_query.CreateNonLinearAlgorithmIndex(
            store=store_payload_no_predicates["store"],
            non_linear_indices=[NonLinearIndex(hnsw=HnswConfig())],
        )
        response = await client.create_non_linear_algorithm_index(create_index_request)
        assert response.created_indexes == 1

        # Drop HNSW index
        drop_index_request = db_query.DropNonLinearAlgorithmIndex(
            store=store_payload_no_predicates["store"],
            non_linear_indices=[NonLinearAlgorithm.HNSW],
            error_if_not_exists=True,
        )
        response = await client.drop_non_linear_algorithm_index(drop_index_request)
        assert response.deleted_count == 1
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_db_pipeline_create_and_query(spin_up_ahnlich_db):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_db)
    client = db_service.DbServiceStub(channel)
    try:
        # Create pipeline request
        pipeline_request = pipeline.DbRequestPipeline(
            queries=[
                pipeline.DbQuery(
                    create_store=db_query.CreateStore(
                        store="PipelineTestStore", dimension=5, error_if_exists=True
                    )
                ),
                pipeline.DbQuery(
                    set=db_query.Set(
                        store="PipelineTestStore",
                        inputs=[
                            keyval.DbStoreEntry(
                                key=keyval.StoreKey(key=[1.0, 2.0, 3.0, 4.0, 5.0]),
                                value=keyval.StoreValue(
                                    value={
                                        "name": metadata.MetadataValue(
                                            raw_string="TestItem"
                                        )
                                    }
                                ),
                            )
                        ],
                    )
                ),
                pipeline.DbQuery(
                    get_key=db_query.GetKey(
                        store="PipelineTestStore",
                        keys=[keyval.StoreKey(key=[1.0, 2.0, 3.0, 4.0, 5.0])],
                    )
                ),
            ]
        )

        response = await client.pipeline(pipeline_request)
        assert len(response.responses) == 3
        assert isinstance(response.responses[0].unit, db_server.Unit)
        assert isinstance(response.responses[1].set, db_server.Set)
        assert isinstance(response.responses[2].get, db_server.Get)
        assert len(response.responses[2].get.entries) == 1
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_db_pipeline_bulk_operations(spin_up_ahnlich_db):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_db)
    client = db_service.DbServiceStub(channel)
    try:
        # Create bulk operations pipeline
        keys = [keyval.StoreKey(key=[float(i)] * 5) for i in range(1, 6)]
        pipeline_request = pipeline.DbRequestPipeline(
            queries=[
                pipeline.DbQuery(
                    create_store=db_query.CreateStore(
                        store="BulkOpsStore", dimension=5, error_if_exists=True
                    )
                ),
                pipeline.DbQuery(
                    set=db_query.Set(
                        store="BulkOpsStore",
                        inputs=[
                            keyval.DbStoreEntry(
                                key=key,
                                value=keyval.StoreValue(
                                    value={
                                        "value": metadata.MetadataValue(
                                            raw_string=f"Item{i+1}"
                                        )
                                    }
                                ),
                            )
                            for i, key in enumerate(keys)
                        ],
                    )
                ),
                pipeline.DbQuery(list_stores=db_query.ListStores()),
                pipeline.DbQuery(
                    del_key=db_query.DelKey(
                        store="BulkOpsStore", keys=keys[:2]  # Delete first two items
                    )
                ),
            ]
        )

        response = await client.pipeline(pipeline_request)
        assert len(response.responses) == 4
        assert response.responses[1].set.upsert.inserted == 5
        assert len(response.responses[2].store_list.stores) == 1
        assert response.responses[3].del_.deleted_count == 2
    finally:
        channel.close()


@pytest.mark.asyncio
async def test_db_pipeline_mixed_success_and_error(spin_up_ahnlich_db):
    channel = Channel(host="127.0.0.1", port=spin_up_ahnlich_db)
    client = db_service.DbServiceStub(channel)
    try:
        # Create pipeline with one valid and one invalid operation
        pipeline_request = pipeline.DbRequestPipeline(
            queries=[
                pipeline.DbQuery(
                    create_store=db_query.CreateStore(
                        store="MixedResultStore", dimension=5, error_if_exists=True
                    )
                ),
                pipeline.DbQuery(
                    get_key=db_query.GetKey(
                        store="NonExistentStore",
                        keys=[keyval.StoreKey(key=[1.0, 2.0, 3.0, 4.0, 5.0])],
                    )
                ),
            ]
        )

        response = await client.pipeline(pipeline_request)
        assert len(response.responses) == 2
        assert response.responses[0].unit is not None
        assert response.responses[1].error is not None
    finally:
        channel.close()
