from ahnlich_client_py.clients import AhnlichAIClient, AhnlichDBClient
from ahnlich_client_py.internals import ai_query
from book_search.split_book import get_book

ai_store_payload_with_predicates = {
    "store_name": "book",
    "query_model": ai_query.AIModel__BGEBaseEnV15(),
    "index_model": ai_query.AIModel__BGEBaseEnV15(),
    "predicates": ["chapter", "paragraph"],
    "error_if_exists": False
}

def run_insert_text():
  ai_client = AhnlichAIClient(address="127.0.0.1", port=1370, connect_timeout_sec=600)
  ai_client.drop_store(
    store_name=ai_store_payload_with_predicates["store_name"],
    error_if_not_exists=False
  )
  store_inputs = get_book()
  cr_respone = ai_client.create_store(**ai_store_payload_with_predicates)
  print(cr_respone)
  for i in range(0, len(store_inputs), 10):
    response = ai_client.set(
      store_name=ai_store_payload_with_predicates["store_name"],
      inputs=[store_inputs[i:i+10]],
      preprocess_action=ai_query.PreprocessAction__RawString(
          ai_query.StringAction__ErrorIfTokensExceed()
      ),
    )
    print(response)


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