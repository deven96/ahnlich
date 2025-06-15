use ahnlich_types::utils::TRACE_HEADER;
use tracing_opentelemetry::OpenTelemetrySpanExt;

pub fn trace_with_parent(req: &http::Request<()>) -> tracing::Span {
    let span = tracing::info_span!("query-processor");
    if let Some(trace_parent) = req
        .headers()
        .get(TRACE_HEADER)
        .and_then(|val| val.to_str().ok())
    {
        if let Ok(parent_context) = tracer::trace_parent_to_span(trace_parent) {
            span.set_parent(parent_context);
        };
    }
    span
}
