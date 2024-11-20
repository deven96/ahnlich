import asyncio
from functools import partial

from ahnlich_client_py.clients.non_blocking import AhnlichAIClient
from ahnlich_client_py.config import AhnlichPoolSettings
from ahnlich_client_py.internals import ai_query

from book_search.split_book import get_book

ai_store_payload_with_predicates = {
    "store_name": "book",
    "query_model": ai_query.AIModel__BGEBaseEnV15(),
    "index_model": ai_query.AIModel__BGEBaseEnV15(),
    "predicates": ["chapter", "paragraph"],
    "error_if_exists": False,
}


async def set_client(ai_client, inputs):
    response = await ai_client.set(
        store_name=ai_store_payload_with_predicates["store_name"],
        inputs=inputs,
        preprocess_action=ai_query.PreprocessAction__RawString(
            ai_query.StringAction__ErrorIfTokensExceed()
        ),
    )

    print(response)


async def create_tasks():
    store_inputs = get_book()
    return [
        partial(set_client, inputs=store_inputs[i : i + 10])
        for i in range(0, len(store_inputs), 10)
    ]


async def run_insert_text(ai_client):
    await ai_client.drop_store(
        store_name=ai_store_payload_with_predicates["store_name"],
        error_if_not_exists=False,
    )
    await ai_client.create_store(**ai_store_payload_with_predicates)

    task_definitions = await create_tasks()
    tasks = [asyncio.create_task(task(ai_client)) for task in task_definitions]

    await asyncio.gather(*tasks)


async def insert_book():
    pool_setting = AhnlichPoolSettings()
    pool_setting.max_pool_size = 35
    ai_client = AhnlichAIClient(
        address="127.0.0.1",
        port=1370,
        connect_timeout_sec=600,
        pool_settings=pool_setting,
    )
    await run_insert_text(ai_client=ai_client)


loop = asyncio.get_event_loop()


def main():
    asyncio.run(insert_book())
