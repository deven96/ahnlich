import asyncio

from functools import partial

from grpclib.client import Channel

from ahnlich_client_py.grpc import keyval, metadata
from ahnlich_client_py.grpc.ai import query as ai_query, preprocess
from ahnlich_client_py.grpc.ai.models import AiModel
from ahnlich_client_py.grpc.services import ai_service

from book_search.split_book import get_book

STORE_NAME = "book"
PREDICATES = ["chapter", "paragraph"]

CREATE_STORE_REQUEST = ai_query.CreateStore(
    store=STORE_NAME,
    query_model=AiModel.BGE_BASE_EN_V15,
    index_model=AiModel.BGE_BASE_EN_V15,
    predicates=PREDICATES,
    error_if_exists=False,
    store_original=True,
)


async def set_batch(client: ai_service.AiServiceStub, entries):
    request = ai_query.Set(
        store=STORE_NAME,
        inputs=entries,
        preprocess_action=preprocess.PreprocessAction.NoPreprocessing,
    )
    response = await client.set(request)
    print(response)


async def create_tasks():
    store_inputs = get_book()
    return [
        partial(set_batch, entries=store_inputs[i : i + 10])
        for i in range(0, len(store_inputs), 10)
    ]


async def insert_book():
    channel = Channel(host="127.0.0.1", port=1370)
    client = ai_service.AiServiceStub(channel)

    try:
        await client.drop_store(
            ai_query.DropStore(store=STORE_NAME, error_if_not_exists=False)
        )
        await client.create_store(CREATE_STORE_REQUEST)

        task_definitions = await create_tasks()

        tasks = [asyncio.create_task(task(client=client)) for task in task_definitions]
        await asyncio.gather(*tasks)
    finally:
        channel.close()


loop = asyncio.get_event_loop()


def main():
    asyncio.run(insert_book())
