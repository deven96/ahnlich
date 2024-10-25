import io
from urllib.request import urlopen

from PIL import Image

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

ai_store_payload_with_predicates_images = {
    "store_name": "Diretnan Image Predication Stores",
    "query_model": ai_query.AIModel__Resnet50(),
    "index_model": ai_query.AIModel__Resnet50(),
    "predicates": ["special", "brand"],
}


def run_insert_text():
    ai_client = AhnlichAIClient(address="127.0.0.1", port=1370, connect_timeout_sec=30)
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


def run_get_simn_text():
    ai_client = AhnlichAIClient(address="127.0.0.1", port=1370)
    builder = ai_client.pipeline()
    builder.get_sim_n(
        store_name=ai_store_payload_with_predicates["store_name"],
        search_input=ai_query.StoreInput__RawString("Basketball"),
        closest_n=3,
        algorithm=ai_query.Algorithm__CosineSimilarity(),
    )
    return builder.exec()


def run_insert_image():
    image_urls = [
        (
            "https://cdn.britannica.com/96/195196-050-3909D5BD/Michael-Jordan-1988.jpg",
            "Slam Dunk Jordan",
        ),
        ("https://i.ebayimg.com/images/g/0-wAAOSwsQ1h5Pqc/s-l1600.webp", "Air Jordan"),
        (
            "https://as2.ftcdn.net/v2/jpg/02/70/86/51/1000_F_270865104_HMpmjP3Hqt0MvdlV7QkQJful50bBzj46.jpg",
            "Aeroplane",
        ),
        (
            "https://csaenvironmental.co.uk/wp-content/uploads/2020/06/landscape-value-600x325.jpg",
            "Landscape",
        ),
    ]

    ai_client = AhnlichAIClient(address="127.0.0.1", port=1370, connect_timeout_sec=30)
    builder = ai_client.pipeline()
    builder.create_store(**ai_store_payload_with_predicates_images)
    for url, brand in image_urls:
        print("Processing image: ", url)
        location = urlopen(url)
        image = Image.open(location)
        buffer = io.BytesIO()
        image.save(buffer, format=image.format)

        store_inputs = [
            (
                ai_query.StoreInput__Image(buffer.getvalue()),
                {"brand": ai_query.MetadataValue__RawString(brand)},
            ),
        ]

        builder.set(
            store_name=ai_store_payload_with_predicates_images["store_name"],
            inputs=store_inputs,
            preprocess_action=ai_query.PreprocessAction__Image(
                ai_query.ImageAction__ResizeImage()
            ),
        )

    return builder.exec()


def run_get_simn_image():
    ai_client = AhnlichAIClient(address="127.0.0.1", port=1370)
    builder = ai_client.pipeline()
    url = "https://i.pinimg.com/564x/9d/76/c8/9d76c8229b7528643d69636c1a9a428d.jpg"
    image = Image.open(urlopen(url))
    buffer = io.BytesIO()
    image.save(buffer, format=image.format)
    builder.get_sim_n(
        store_name=ai_store_payload_with_predicates_images["store_name"],
        search_input=ai_query.StoreInput__Image(buffer.getvalue()),
        closest_n=3,
        algorithm=ai_query.Algorithm__CosineSimilarity(),
    )
    return builder.exec()
