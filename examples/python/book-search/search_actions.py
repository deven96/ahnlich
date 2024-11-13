import asyncio
from concurrent.futures import Future, ThreadPoolExecutor
from functools import partial
from typing import List

from ahnlich_client_py.clients import AhnlichAIClient
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

pool_setting = AhnlichPoolSettings()
pool_setting.max_pool_size = 8
ai_client = AhnlichAIClient(
    address="127.0.0.1", port=1370, connect_timeout_sec=600, pool_settings=pool_setting
)


def set_client(inputs):
    response = ai_client.set(
        store_name=ai_store_payload_with_predicates["store_name"],
        inputs=inputs,
        preprocess_action=ai_query.PreprocessAction__RawString(
            ai_query.StringAction__ErrorIfTokensExceed()
        ),
    )

    print(response)


def create_tasks():
    store_inputs = get_book()
    return [
        partial(set_client, store_inputs[i : i + 10])
        for i in range(0, len(store_inputs), 10)
    ]


def insert_book():
    ai_client.drop_store(
        store_name=ai_store_payload_with_predicates["store_name"],
        error_if_not_exists=False,
    )
    ai_client.create_store(**ai_store_payload_with_predicates)

    task_definitions = create_tasks()
    thread_tasks: List[Future] = []
    with ThreadPoolExecutor(max_workers=30) as executor:
        for partial_func in task_definitions:
            thread_tasks.append(executor.submit(partial_func))

        for task in thread_tasks:
            try:
                _ = task.result()
            except Exception as exc:
                print(f"Exception gotten from task {task}... {exc}")

        print("Cleaning up Connection Pool...")
        ai_client.cleanup()
        print("Shut down!....")


def run_get_simn_text(input_query):
    ai_client = AhnlichAIClient(address="127.0.0.1", port=1370)
    builder = ai_client.pipeline()
    builder.get_sim_n(
        store_name=ai_store_payload_with_predicates["store_name"],
        search_input=ai_query.StoreInput__RawString(input_query),
        closest_n=5,
        algorithm=ai_query.Algorithm__CosineSimilarity(),
    )
    return builder.exec()


def search_phrase():
    input_query = input("Please enter the phrase: ")
    response = run_get_simn_text(input_query)
    for result in response.results[0].value.value:
        print(f'Chapter {result[1]["chapter"].value}')
        print(f'Paragraph {result[1]["paragraph"].value}')
        print(result[0].value)
        print("\n")

    ai_client.cleanup()
