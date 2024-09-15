from ahnlich_client_py.clients import AhnlichAIClient
from ahnlich_client_py.internals import ai_query

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


def run_insert():
    ai_client = AhnlichAIClient(address="127.0.0.1", port=1370)
    store_inputs = [
        (
            ai_query.StoreInput__RawString("Jordan One"),
            {"brand": ai_query.MetadataValue__RawString("Nike")},
        ),
        (
            ai_query.StoreInput__RawString("Air Jordan"),
            {"brand": ai_query.MetadataValue__RawString("Nike")},
        ),
        (
            ai_query.StoreInput__RawString("Chicago Bulls"),
            {"brand": ai_query.MetadataValue__RawString("NBA")},
        ),
        (
            ai_query.StoreInput__RawString("Los Angeles Lakers"),
            {"brand": ai_query.MetadataValue__RawString("NBA")},
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
    return builder.exec()


def run_get_simn():
    ai_client = AhnlichAIClient(address="127.0.0.1", port=1370)
    builder = ai_client.pipeline()
    builder.get_sim_n(
        store_name=ai_store_payload_with_predicates["store_name"],
        search_input=ai_query.StoreInput__RawString("Basketball"),
        closest_n=3,
        algorithm=ai_query.Algorithm__CosineSimilarity(),
    )
    return builder.exec()
