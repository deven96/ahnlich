import asyncio
import io
import os
from urllib.request import urlopen

from grpclib.client import Channel
from opentelemetry import trace
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.sdk.resources import SERVICE_NAME, Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor
from PIL import Image

from ahnlich_client_py import TRACE_HEADER
from ahnlich_client_py.grpc import keyval, metadata
from ahnlich_client_py.grpc.ai import pipeline, preprocess
from ahnlich_client_py.grpc.ai import query as ai_query
from ahnlich_client_py.grpc.ai.execution_provider import ExecutionProvider
from ahnlich_client_py.grpc.ai.models import AiModel
from ahnlich_client_py.grpc.algorithm.algorithms import Algorithm
from ahnlich_client_py.grpc.services import ai_service


class Text2TextDemo:
    def __init__(self, span_id: str | None = None):
        self.query_model = AiModel.ALL_MINI_LM_L6_V2
        self.index_model = AiModel.ALL_MINI_LM_L6_V2
        self.store_name = "The Sports Press Club"
        self.predicates = ["sport"]

        self.channel = Channel(host="127.0.0.1", port=1370)
        self.client = ai_service.AiServiceStub(self.channel)
        self.metadata: dict[str, str] | None = (
            {TRACE_HEADER: span_id} if span_id else None
        )

    async def close(self):
        self.channel.close()

    async def insert(self):
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

        entries = [
            ai_query.StoreEntry(
                key=keyval.StoreInput(raw_string=snippet),
                value={"sport": metadata.MetadataValue(raw_string=sport)},
            )
            for snippet, sport in snippets_and_sports
        ]

        pipeline_request = pipeline.AiRequestPipeline(
            queries=[
                pipeline.AiQuery(
                    create_store=ai_query.CreateStore(
                        store=self.store_name,
                        query_model=self.query_model,
                        index_model=self.index_model,
                        predicates=self.predicates,
                        error_if_exists=True,
                        store_original=True,
                    )
                ),
                pipeline.AiQuery(
                    set=ai_query.Set(
                        store=self.store_name,
                        inputs=entries,
                        preprocess_action=preprocess.PreprocessAction.ModelPreprocessing,
                    )
                ),
            ]
        )
        return await self.client.pipeline(pipeline_request, metadata=self.metadata)

    async def query(self):
        # Step 4: Run similarity query
        response = await self.client.get_sim_n(
            ai_query.GetSimN(
                store=self.store_name,
                search_input=keyval.StoreInput(
                    raw_string="News events where athletes broke a record"
                ),
                closest_n=3,
                algorithm=Algorithm.CosineSimilarity,
            ),
            metadata=self.metadata,
        )
        return response


class VeryShortText2TextDemo:
    def __init__(self, span_id: str | None = None):
        self.query_model = AiModel.CLIP_VIT_B32_TEXT
        self.index_model = AiModel.CLIP_VIT_B32_TEXT
        self.store_name = "The Literary Collection"
        self.predicates = ["citizenship"]

        self.channel = Channel(host="127.0.0.1", port=1370)
        self.client = ai_service.AiServiceStub(self.channel)
        self.metadata: dict[str, str] | None = (
            {TRACE_HEADER: span_id} if span_id else None
        )

    async def close(self):
        self.channel.close()

    async def insert(self):
        snippets_and_citizenship = [
            ("1984", "English"),
            ("Things Fall Apart", "Nigerian"),
            ("The Great Gatsby", "American"),
            ("The Alchemist", "Brazilian"),
            ("Man's Search for Meaning", "Austrian"),
        ]

        entries = [
            ai_query.StoreEntry(
                key=keyval.StoreInput(raw_string=snippet),
                value={"citizenship": metadata.MetadataValue(raw_string=citizenship)},
            )
            for snippet, citizenship in snippets_and_citizenship
        ]

        pipeline_request = pipeline.AiRequestPipeline(
            queries=[
                pipeline.AiQuery(
                    create_store=ai_query.CreateStore(
                        store=self.store_name,
                        query_model=self.query_model,
                        index_model=self.index_model,
                        predicates=self.predicates,
                        error_if_exists=True,
                        store_original=True,
                    )
                ),
                pipeline.AiQuery(
                    set=ai_query.Set(
                        store=self.store_name,
                        inputs=entries,
                        preprocess_action=preprocess.PreprocessAction.ModelPreprocessing,
                    )
                ),
            ]
        )
        return await self.client.pipeline(pipeline_request, metadata=self.metadata)

    async def query(self):

        response = await self.client.get_sim_n(
            ai_query.GetSimN(
                store=self.store_name,
                search_input=keyval.StoreInput(raw_string="Chinua Achebe"),
                closest_n=3,
                algorithm=Algorithm.CosineSimilarity,
            ),
            metadata=self.metadata,
        )
        return response


def url_to_buffer(url) -> bytes:
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
    return buffer.getvalue()


class Text2ImageDemo:
    def __init__(self, span_id: str | None = None):
        self.query_model = AiModel.CLIP_VIT_B32_TEXT
        self.index_model = AiModel.CLIP_VIT_B32_IMAGE
        self.store_name = "The Sports Image Collection"
        self.predicates = ["athlete"]

        self.channel = Channel(host="127.0.0.1", port=1370)
        self.client = ai_service.AiServiceStub(self.channel)

        self.metadata: dict[str, str] | None = (
            {TRACE_HEADER: span_id} if span_id else None
        )

    async def close(self):
        self.channel.close()

    async def insert(self):

        image_urls_and_athletes = [
            [
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
            ],
        ]

        # Process images and create store_inputs

        entries = [
            ai_query.StoreEntry(
                key=keyval.StoreInput(image=url_to_buffer(url)),
                value={"brand": metadata.MetadataValue(raw_string=name)},
            )
            for url, name in image_urls_and_athletes
        ]

        pipeline_request = pipeline.AiRequestPipeline(
            queries=[
                pipeline.AiQuery(
                    create_store=ai_query.CreateStore(
                        store=self.store_name,
                        query_model=self.query_model,
                        index_model=self.index_model,
                        predicates=self.predicates,
                        store_original=False,
                        error_if_exists=True,
                    )
                ),
                pipeline.AiQuery(
                    set=ai_query.Set(
                        store=self.store_name,
                        inputs=entries,
                        preprocess_action=preprocess.PreprocessAction.ModelPreprocessing,
                    )
                ),
            ]
        )
        return await self.client.pipeline(pipeline_request, metadata=self.metadata)

    async def query(self):
        response = await self.client.get_sim_n(
            ai_query.GetSimN(
                store=self.store_name,
                search_input=keyval.StoreInput(raw_string="United States vs England"),
                closest_n=3,
                algorithm=Algorithm.CosineSimilarity,
            ),
            metadata=self.metadata,
        )
        return response


class Image2ImageDemo:
    def __init__(self, span_id: str | None):
        self.query_model = AiModel.CLIP_VIT_B32_IMAGE
        self.index_model = AiModel.CLIP_VIT_B32_IMAGE
        self.store_name = "The Jordan or Not Jordan Collection"
        self.predicates = ["label"]
        self.channel = Channel(host="127.0.0.1", port=1370)
        self.client = ai_service.AiServiceStub(self.channel)
        self.metadata: dict[str, str] | None = (
            {TRACE_HEADER: span_id} if span_id else None
        )

    async def close(self):
        self.channel.close()

    async def insert(self):

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

        entries = [
            ai_query.StoreEntry(
                key=keyval.StoreInput(image=url_to_buffer(url)),
                value={"label": metadata.MetadataValue(raw_string=label)},
            )
            for url, label in image_urls_and_labels
        ]

        pipeline_request = pipeline.AiRequestPipeline(
            queries=[
                pipeline.AiQuery(
                    create_store=ai_query.CreateStore(
                        store=self.store_name,
                        query_model=self.query_model,
                        index_model=self.index_model,
                        predicates=self.predicates,
                        store_original=False,
                        error_if_exists=True,
                    )
                ),
                pipeline.AiQuery(
                    set=ai_query.Set(
                        store=self.store_name,
                        inputs=entries,
                        preprocess_action=preprocess.PreprocessAction.ModelPreprocessing,
                        execution_provider=ExecutionProvider.CUDA,
                    )
                ),
            ]
        )
        return await self.client.pipeline(pipeline_request, metadata=self.metadata)

    async def query(self):
        # Query with an image
        query_url = (
            "https://i.pinimg.com/564x/9d/76/c8/9d76c8229b7528643d69636c1a9a428d.jpg"
        )
        image_bytes = url_to_buffer(query_url)

        response = await self.client.get_sim_n(
            ai_query.GetSimN(
                store=self.store_name,
                search_input=keyval.StoreInput(image=image_bytes),
                closest_n=3,
                algorithm=Algorithm.CosineSimilarity,
            ),
            metadata=self.metadata,
        )

        return response


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


async def run_with_tracing():
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
        result = await demo.insert()
        print(result)
        await demo.close()


if __name__ == "__main__":
    asyncio.get_event_loop().run_until_complete(run_with_tracing())
