from ahnlich_client_py.clients import AhnlichAIClient
from ahnlich_client_py.internals import ai_query


def run_get_simn_text(input_query):
    ai_client = AhnlichAIClient(address="127.0.0.1", port=1370)
    builder = ai_client.pipeline()
    builder.get_sim_n(
        store_name="book",
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
