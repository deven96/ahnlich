import io
import os
from urllib.request import urlopen

from opentelemetry import trace
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.sdk.resources import SERVICE_NAME, Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor
from PIL import Image

from ahnlich_client_py.clients import AhnlichAIClient
from ahnlich_client_py.internals import ai_query


class Text2TextDemo:
    def __init__(self):
        ai_client = AhnlichAIClient(
            address="127.0.0.1", port=1370, connect_timeout_sec=30
        )
        self.query_model = ai_query.AIModel__AllMiniLML6V2()
        self.index_model = ai_query.AIModel__AllMiniLML6V2()
        self.store_name = "The Sports Press Club"
        self.builder = ai_client.pipeline()
        predicates = ["sport"]
        self.builder.create_store(
            store_name=self.store_name,
            query_model=self.query_model,
            index_model=self.index_model,
            predicates=predicates,
        )

    def insert(self):
        # Initial list of tuples (snippet, sport)
        snippets_and_sports = [
            (
                "Manchester City secures a thrilling 2-1 victory over Liverpool in the Premier League, "
                "with Erling Haaland scoring the decisive goal in the 87th minute.",
                "Football",
            ),
            (
                "Coco Gauff clinches a hard-fought victory in a gripping three-set final against Iga Swiatek "
                "to win the Wimbledon Finals, solidifying her place among the top competitors.",
                "Tennis",
            ),
            (
                "LeBron James makes history yet again, becoming the NBA's all-time leading scorer in a single "
                "season as the Lakers defeat the Golden State Warriors 120-115.",
                "Basketball",
            ),
            (
                "India edges out Australia in a nail-biting T20 match, with Virat Kohli's unbeaten 78 "
                "guiding the team to a thrilling last-over victory.",
                "Cricket",
            ),
            (
                "Max Verstappen dominates the Abu Dhabi Grand Prix, achieving an incredible 16th win "
                "of the season, a milestone that underscores his unparalleled dominance and secures his third "
                "consecutive championship title.",
                "Formula 1",
            ),
        ]

        store_inputs = [
            (
                ai_query.StoreInput__RawString(snippet),
                {"sport": ai_query.MetadataValue__RawString(sport)},
            )
            for snippet, sport in snippets_and_sports
        ]

        self.builder.set(
            store_name=self.store_name,
            inputs=store_inputs,
            preprocess_action=ai_query.PreprocessAction__ModelPreprocessing(),
        )
        return self.builder.exec()

    def query(self):
        search_input = "News events where athletes broke a record"
        self.builder.get_sim_n(
            store_name=self.store_name,
            search_input=ai_query.StoreInput__RawString(search_input),
            closest_n=3,
            algorithm=ai_query.Algorithm__CosineSimilarity(),
        )
        return self.builder.exec()


class VeryShortText2TextDemo:
    def __init__(self):
        ai_client = AhnlichAIClient(
            address="127.0.0.1", port=1370, connect_timeout_sec=30
        )
        self.query_model = ai_query.AIModel__ClipVitB32Text()
        self.index_model = ai_query.AIModel__ClipVitB32Text()
        self.store_name = "The Literary Collection"
        self.builder = ai_client.pipeline()
        predicates = ["citizenship"]
        self.builder.create_store(
            store_name=self.store_name,
            query_model=self.query_model,
            index_model=self.index_model,
            predicates=predicates,
        )

    def insert(self):
        # Initial list of tuples (snippet, writer's citizenship)
        snippets_and_citizenship = [
            ("1984", "English"),
            ("Things Fall Apart", "Nigerian"),
            ("The Great Gatsby", "American"),
            ("The Alchemist", "Brazilian"),
            ("Man's Search for Meaning", "Austrian"),
        ]

        # Create store_inputs using a list comprehension
        store_inputs = [
            (
                ai_query.StoreInput__RawString(snippet),
                {"citizenship": ai_query.MetadataValue__RawString(citizenship)},
            )
            for snippet, citizenship in snippets_and_citizenship
        ]

        self.builder.set(
            store_name=self.store_name,
            inputs=store_inputs,
            preprocess_action=ai_query.PreprocessAction__ModelPreprocessing(),
        )
        return self.builder.exec()

    def query(self):
        search_input = "Chinua Achebe"
        self.builder.get_sim_n(
            store_name=self.store_name,
            search_input=ai_query.StoreInput__RawString(search_input),
            closest_n=3,
            algorithm=ai_query.Algorithm__CosineSimilarity(),
        )
        return self.builder.exec()


def url_to_buffer(url):
    """
    Converts an image URL or local file path to a buffer value.
    :param url: URL or file path of the image.
    :return: BytesIO buffer containing the image data.
    """
    print(f"Processing image: {url}")
    if url.startswith("http"):
        location = urlopen(url)
    else:
        location = url

    image = Image.open(location)
    buffer = io.BytesIO()
    image.save(buffer, format=image.format)
    buffer.seek(0)  # Reset the buffer pointer to the beginning
    return buffer


class Text2ImageDemo:
    def __init__(self):
        ai_client = AhnlichAIClient(
            address="127.0.0.1", port=1370, connect_timeout_sec=30
        )
        self.query_model = ai_query.AIModel__ClipVitB32Text()
        self.index_model = ai_query.AIModel__ClipVitB32Image()
        self.store_name = "The Sports Image Collection"
        self.builder = ai_client.pipeline()
        predicates = ["athlete"]
        self.builder.create_store(
            store_name=self.store_name,
            query_model=self.query_model,
            index_model=self.index_model,
            predicates=predicates,
            store_original=False,
        )

    def insert(self):
        # Initial list of tuples (image URL, athlete name)
        image_urls_and_athletes = [
            (
                "https://imageio.forbes.com/specials-images/imageserve/632357fbf1cebc1639065099/Roger-Federer-celebrated"
                "-after-beating-Lorenzo-Sonego-at-Wimbledon-last-year-/1960x0.jpg?format=jpg&width=960",
                "Roger Federer",
            ),
            (
                "https://www.silverarrows.net/wp-content/uploads/2020/05/Lewis-Hamilton-Japan.jpg",
                "Lewis Hamilton",
            ),
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

        # Process images and create store_inputs
        store_inputs = [
            (
                ai_query.StoreInput__Image(url_to_buffer(url).getvalue()),
                {"brand": ai_query.MetadataValue__RawString(athlete)},
            )
            for url, athlete in image_urls_and_athletes
        ]

        # Set the store inputs
        self.builder.set(
            store_name=self.store_name,
            inputs=store_inputs,
            preprocess_action=ai_query.PreprocessAction__ModelPreprocessing(),
        )
        return self.builder.exec()

    def query(self):
        search_input = "United States vs England"
        self.builder.get_sim_n(
            store_name=self.store_name,
            search_input=ai_query.StoreInput__RawString(search_input),
            closest_n=3,
            algorithm=ai_query.Algorithm__CosineSimilarity(),
        )
        return self.builder.exec()


class Image2ImageDemo:
    def __init__(self, span_id):
        ai_client = AhnlichAIClient(
            address="127.0.0.1", port=1370, connect_timeout_sec=30
        )
        self.query_model = ai_query.AIModel__ClipVitB32Image()
        self.index_model = ai_query.AIModel__ClipVitB32Image()
        self.store_name = "The Jordan or Not Jordan Collection"
        self.builder = ai_client.pipeline(span_id)
        predicates = ["label"]
        self.builder.create_store(
            store_name=self.store_name,
            query_model=self.query_model,
            index_model=self.index_model,
            predicates=predicates,
            store_original=False,
        )

    def insert(self):
        # Initial list of tuples (image URL, image label)
        image_urls_and_labels = [
            (
                "https://cdn.britannica.com/96/195196-050-3909D5BD/Michael-Jordan-1988.jpg",
                "Slam Dunk Jordan",
            ),
            (
                "https://i.ebayimg.com/images/g/0-wAAOSwsQ1h5Pqc/s-l1600.webp",
                "Air Jordan",
            ),
            (
                "https://as2.ftcdn.net/v2/jpg/02/70/86/51/1000_F_270865104_HMpmjP3Hqt0MvdlV7QkQJful50bBzj46.jpg",
                "Aeroplane",
            ),
            (
                "https://csaenvironmental.co.uk/wp-content/uploads/2020/06/landscape-value-600x325.jpg",
                "Landscape",
            ),
            (
                "https://images2.minutemediacdn.com/image/upload/images%2FGettyImages%2Fmmsport%2F29%2F01j9hmvteb5pzsx00tgp.jpg",
                "Victor Wembanyama blocks Lebron",
            ),
        ]

        # Process images and create store_inputs
        store_inputs = [
            (
                ai_query.StoreInput__Image(url_to_buffer(url).getvalue()),
                {"label": ai_query.MetadataValue__RawString(label)},
            )
            for url, label in image_urls_and_labels
        ]

        # Set the store inputs
        self.builder.set(
            store_name=self.store_name,
            inputs=store_inputs,
            preprocess_action=ai_query.PreprocessAction__ModelPreprocessing(),
            execution_provider=ai_query.ExecutionProvider__CUDA(),
        )
        return self.builder.exec()

    def query(self):
        # Query with an image
        query_url = (
            "https://i.pinimg.com/564x/9d/76/c8/9d76c8229b7528643d69636c1a9a428d.jpg"
        )
        buffer = url_to_buffer(query_url)

        self.builder.get_sim_n(
            store_name=self.store_name,
            search_input=ai_query.StoreInput__Image(buffer.getvalue()),
            closest_n=3,
            algorithm=ai_query.Algorithm__CosineSimilarity(),
        )
        return self.builder.exec()


def setup_tracing():
    # Step 1: Create a Resource with the service name
    resource = Resource(attributes={SERVICE_NAME: "ahnlich_python_client"})

    # Step 2: Initialize the Tracer Provider with the resource
    trace.set_tracer_provider(TracerProvider(resource=resource))
    # # Step 3: Initialize the Tracer Provider
    # trace.set_tracer_provider(TracerProvider())

    url = os.getenv("DEMO_OTEL_URL", "http://localhost:4317")
    # Step 4: Configure the OTLP Exporter
    otlp_exporter = OTLPSpanExporter(endpoint=url, insecure=True)

    # Step 5: Add the Span Processor to the Tracer Provider
    span_processor = BatchSpanProcessor(otlp_exporter)
    trace.get_tracer_provider().add_span_processor(span_processor)

    # Step 6: Get a Tracer
    trace_obj = trace.get_tracer("ahnlich_python_client")
    return trace_obj


def run_with_tracing():
    print("[INFO] Running tracing")
    with setup_tracing().start_as_current_span("info_span") as span:
        span.set_attribute("data-application", "ahnlich_client_py")
        span.add_event(
            "Testing spanning",
            {"log.severity": "INFO", "log.message": "This is an info-level log."},
        )
        span_context = span.get_span_context()
        trace_parent_id = "00-{:032x}-{:016x}-{:02x}".format(
            span_context.trace_id, span_context.span_id, span_context.trace_flags
        )
        demo = Image2ImageDemo(trace_parent_id)
        a = demo.insert()
        print(a)


if __name__ == "__main__":
    run_with_tracing()
