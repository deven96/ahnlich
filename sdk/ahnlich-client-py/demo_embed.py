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

ai_store_payload_with_predicates_images_texts = {
    "store_name": "Diretnan Image Text Predication Stores",
    "query_model": ai_query.AIModel__ClipVitB32Text(),
    "index_model": ai_query.AIModel__ClipVitB32Image(),
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


def insert_image(urls, store_data):
    ai_client = AhnlichAIClient(address="127.0.0.1", port=1370, connect_timeout_sec=30)
    builder = ai_client.pipeline()
    builder.create_store(**store_data)
    for url, brand in urls:
        print("Processing image: ", url)
        if url.startswith("http"):
            location = urlopen(url)
        else:
            location = url
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
            store_name=store_data["store_name"],
            inputs=store_inputs,
            preprocess_action=ai_query.PreprocessAction__Image(
                ai_query.ImageAction__ResizeImage()
            ),
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
    return insert_image(image_urls, ai_store_payload_with_predicates_images)


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


def run_insert_image_text():
    image_urls = [
        (
            "https://imageio.forbes.com/specials-images/imageserve/632357fbf1cebc1639065099/Roger-Federer-celebrated"
            "-after-beating-Lorenzo-Sonego-at-Wimbledon-last-year-/1960x0.jpg?format=jpg&width=960",
            "Roger Federer",
        ),
        ("https://www.silverarrows.net/wp-content/uploads/2020/05/Lewis-Hamilton-Japan.jpg", "Lewis Hamilton"),
        (
            "https://img.20mn.fr/B2Dto_H3RveJTzabY4IR2yk/1444x920_andreja-laski-of-team-slovenia-and-clarisse-agbegnenou"
            "-team-france-compete-during-the-women-63-kg-semifinal-of-table-b-contest-on-day-four-of-the-olympic-games-"
            "paris-2024-at-champs-de-mars-arena-03vulaurent-d2317-credit-laurent-vu-sipa-2407301738",
            "Clarisse Agbegnenou",
        ),
        (
            "https://c8.alamy.com/comp/R1YEE4/london-uk-15th-november-2018-jadon-sancho-of-england-is-tackled-by-"
            "christian-pulisic-of-usa-during-the-international-friendly-match-between-england-and-usa-at-wembley-"
            "stadium-on-november-15th-2018-in-london-england-photo-by-matt-bradshawphcimages-credit-phc-imagesalamy-live-news-R1YEE4.jpg",
            "Christian Pulisic and Sancho",
        ),
    ]
    return insert_image(image_urls, ai_store_payload_with_predicates_images_texts)


def run_get_simn_image_text():
    ai_client = AhnlichAIClient(address="127.0.0.1", port=1370)
    builder = ai_client.pipeline()
    builder.get_sim_n(
        store_name=ai_store_payload_with_predicates_images_texts["store_name"],
        search_input=ai_query.StoreInput__RawString("United States vs England"),
        closest_n=3,
        algorithm=ai_query.Algorithm__CosineSimilarity(),
    )
    return builder.exec()
