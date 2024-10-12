import os
import traceback

from opentelemetry import trace
from opentelemetry.exporter.otlp.proto.grpc.trace_exporter import \
    OTLPSpanExporter
from opentelemetry.sdk.resources import SERVICE_NAME, Resource
from opentelemetry.sdk.trace import TracerProvider
from opentelemetry.sdk.trace.export import BatchSpanProcessor

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


def tracer(span_id):
    ai_client = AhnlichAIClient(address="127.0.0.1", port=1370)
    store_inputs = [
        (
            ai_query.StoreInput__RawString("Jordan One"),
            {"brand": ai_query.MetadataValue__RawString("Nike")},
        ),
        (
            ai_query.StoreInput__RawString("Yeezey"),
            {"brand": ai_query.MetadataValue__RawString("Adidas")},
        ),
    ]
    builder = ai_client.pipeline(span_id)
    builder.create_store(**ai_store_payload_with_predicates)
    builder.set(
        store_name=ai_store_payload_with_predicates["store_name"],
        inputs=store_inputs,
        preprocess_action=ai_query.PreprocessAction__RawString(
            ai_query.StringAction__ErrorIfTokensExceed()
        ),
    )
    builder.create_store(**ai_store_payload_no_predicates)
    builder.list_stores()
    builder.create_pred_index(
        ai_store_payload_no_predicates["store_name"],
        predicates=["super_sales", "testing", "no mass"],
    )
    builder.drop_pred_index(
        ai_store_payload_no_predicates["store_name"],
        ["testing"],
        error_if_not_exists=True,
    )
    builder.drop_pred_index(
        ai_store_payload_no_predicates["store_name"],
        ["fake_predicate"],
        error_if_not_exists=True,
    )
    try:
        builder.exec()
        ai_client.get_pred(
            ai_store_payload_with_predicates["store_name"],
            ai_query.PredicateCondition__Value(
                value=ai_query.Predicate__Equals(
                    key="brand", value=ai_query.MetadataValue__RawString("Nike")
                )
            ),
        )
    except Exception as exc:
        print(traceback.format_exc())
    finally:
        ai_client.cleanup()


def run_tracing():
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
        tracer(span_id=trace_parent_id)


if __name__ == "__main__":
    run_tracing()
