import asyncio
import os
import random
import string
import traceback

from grpclib.client import Channel
from opentelemetry import trace
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import OTLPSpanExporter
from opentelemetry.sdk.resources import SERVICE_NAME, Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor

from ahnlich_client_py import TRACE_HEADER
from ahnlich_client_py.grpc import keyval, metadata, predicates
from ahnlich_client_py.grpc.ai import pipeline, preprocess
from ahnlich_client_py.grpc.ai import query as ai_query
from ahnlich_client_py.grpc.ai.models import AiModel
from ahnlich_client_py.grpc.services import ai_service


def setup_tracing():
    resource = Resource(attributes={SERVICE_NAME: "ahnlich_python_client"})
    trace.set_tracer_provider(TracerProvider(resource=resource))
    url = os.getenv("DEMO_OTEL_URL", "http://localhost:4317")
    otlp_exporter = OTLPSpanExporter(endpoint=url, insecure=True)
    span_processor = BatchSpanProcessor(otlp_exporter)
    trace.get_tracer_provider().add_span_processor(span_processor)
    return trace.get_tracer("ahnlich_python_client")


def generate_string(length):
    return "".join(random.choices(string.ascii_letters + string.digits, k=length))


def generate_store_inputs(n, text_len):
    return [
        (
            keyval.StoreInput(raw_string=generate_string(text_len)),
            keyval.StoreValue(
                value={"brand": metadata.MetadataValue(raw_string="Nike")}
            ),
        )
        for _ in range(n)
    ]


class AhnlichTracingDemo:
    def __init__(self, span_id: str | None = None):
        self.metadata: dict[str, str] | None = (
            {TRACE_HEADER: span_id} if span_id else None
        )
        self.channel = Channel(host="127.0.0.1", port=1370)
        self.client = ai_service.AiServiceStub(self.channel)
        self.payload_with_pred = ai_query.CreateStore(
            store="Diretnan Predication Stores",
            query_model=AiModel.ALL_MINI_LM_L6_V2,
            index_model=AiModel.ALL_MINI_LM_L6_V2,
            predicates=["special", "brand"],
            error_if_exists=True,
            store_original=True,
        )
        self.payload_no_pred = ai_query.CreateStore(
            store="Diretnan Stores",
            query_model=AiModel.ALL_MINI_LM_L6_V2,
            index_model=AiModel.ALL_MINI_LM_L6_V2,
            error_if_exists=True,
            store_original=True,
        )

    async def close(self):
        self.channel.close()

    async def run(self):
        entries = [
            keyval.AiStoreEntry(key=input_, value=value)
            for input_, value in generate_store_inputs(100, 16)
        ]

        builder = pipeline.AiRequestPipeline(
            queries=[
                pipeline.AiQuery(create_store=self.payload_with_pred),
                pipeline.AiQuery(
                    set=ai_query.Set(
                        store=self.payload_with_pred.store,
                        inputs=entries,
                        preprocess_action=preprocess.PreprocessAction.NoPreprocessing,
                    )
                ),
                pipeline.AiQuery(create_store=self.payload_no_pred),
                pipeline.AiQuery(list_stores=ai_query.ListStores()),
                pipeline.AiQuery(
                    create_pred_index=ai_query.CreatePredIndex(
                        store=self.payload_no_pred.store,
                        predicates=["super_sales", "testing", "no mass"],
                    )
                ),
                pipeline.AiQuery(
                    drop_pred_index=ai_query.DropPredIndex(
                        store=self.payload_no_pred.store,
                        predicates=["testing"],
                        error_if_not_exists=True,
                    )
                ),
                pipeline.AiQuery(
                    drop_pred_index=ai_query.DropPredIndex(
                        store=self.payload_no_pred.store,
                        predicates=["fake_predicate"],
                        error_if_not_exists=True,
                    )
                ),
            ]
        )

        try:
            await self.client.pipeline(builder, metadata=self.metadata)

            response = await self.client.get_pred(
                ai_query.GetPred(
                    store=self.payload_with_pred.store,
                    condition=predicates.PredicateCondition(
                        value=predicates.Predicate(
                            equals=predicates.Equals(
                                key="brand",
                                value=metadata.MetadataValue(raw_string="Nike"),
                            )
                        )
                    ),
                ),
                metadata=self.metadata,
            )

            print(response)
        except Exception:
            print(traceback.format_exc())
        finally:
            await self.close()


async def run_tracing():
    print("[INFO] Running tracing")
    tracer = setup_tracing()
    with tracer.start_as_current_span("info_span") as span:
        span.set_attribute("data-application", "ahnlich_client_py")
        span.add_event(
            "Testing spanning",
            {"log.severity": "INFO", "log.message": "This is an info-level log."},
        )
        span_context = span.get_span_context()
        trace_parent_id = "00-{:032x}-{:016x}-{:02x}".format(
            span_context.trace_id, span_context.span_id, span_context.trace_flags
        )
        demo = AhnlichTracingDemo(span_id=trace_parent_id)
        await demo.run()


if __name__ == "__main__":
    asyncio.run(run_tracing())
