from ahnlich_client_py.clients import AhnlichAIClient
from ahnlich_client_py.internals import ai_query
from ahnlich_client_py.config import AhnlichPoolSettings
from functools import partial

from book_search.split_book import get_book
import asyncio

ai_store_payload_with_predicates = {
    "store_name": "book",
    "query_model": ai_query.AIModel__BGEBaseEnV15(),
    "index_model": ai_query.AIModel__BGEBaseEnV15(),
    "predicates": ["chapter", "paragraph"],
    "error_if_exists": False
}

pool_setting = AhnlichPoolSettings()
pool_setting.max_pool_size = 35
ai_client = AhnlichAIClient(address="127.0.0.1", port=1370, connect_timeout_sec=600, pool_settings=pool_setting)

async def set_client(inputs):
  response = ai_client.set(
    store_name=ai_store_payload_with_predicates["store_name"],
    inputs=inputs,
    preprocess_action=ai_query.PreprocessAction__RawString(
      ai_query.StringAction__ErrorIfTokensExceed()
    ),
  )

  print(response)


async def create_tasks():
  store_inputs = get_book()
  return [partial(set_client, store_inputs[i:i+10]) for i in range(0, len(store_inputs), 10)]

async def run_insert_text():
  
  ai_client.create_store(**ai_store_payload_with_predicates)
  task_definitions = await create_tasks()
  tasks = [asyncio.create_task(task()) for task in task_definitions]
  
  await asyncio.gather(*tasks)

def insert_book():
  ai_client.drop_store(
    store_name=ai_store_payload_with_predicates["store_name"],
    error_if_not_exists=False
  )
  asyncio.run(run_insert_text())

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
    print('\n')