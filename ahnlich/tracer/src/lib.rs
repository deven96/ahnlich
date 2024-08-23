use std::collections::HashMap;

use opentelemetry::{global, trace::TraceContextExt, Context, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    propagation::TraceContextPropagator,
    trace::{self, Sampler},
    Resource,
};
use tracing::subscriber::set_global_default;
use tracing_opentelemetry::OpenTelemetrySpanExt;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

pub fn init_tracing(service_name: &'static str, log_level: Option<&str>, otel_url: &str) {
    let env_filter = EnvFilter::new(log_level.unwrap_or("info"));

    let otel_layer = tracing_opentelemetry::layer().with_tracer(
        opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(otel_url),
            )
            .with_trace_config(
                trace::config()
                    .with_sampler(Sampler::AlwaysOn)
                    .with_resource(Resource::new(vec![KeyValue::new(
                        "service.name",
                        service_name,
                    )])),
            )
            .install_batch(opentelemetry_sdk::runtime::TokioCurrentThread)
            .expect("could not build otel pipeline"),
    );

    let stdout_layer = tracing_subscriber::fmt::layer().pretty();
    let json_layer = tracing_subscriber::fmt::layer()
        .json()
        .with_level(true)
        .with_current_span(true)
        .with_thread_names(true);

    let subscriber = Registry::default().with(env_filter).with(json_layer);

    set_global_default(subscriber.with(stdout_layer).with(otel_layer))
        .expect("Failed to set default subscriber");
    global::set_text_map_propagator(TraceContextPropagator::new());
}

pub fn shutdown_tracing() {
    global::shutdown_tracer_provider();
}

const TRACING_VERSION: u8 = 00;
pub fn span_to_trace_parent(span: tracing::Span) -> Option<String> {
    let otel_context = span.context();
    let span_ref = otel_context.span();
    let span_context = span_ref.span_context();

    if span_context.is_valid() {
        let trace_parent = format!(
            "{:02x}-{:032x}-{:016x}-{:02x}",
            TRACING_VERSION,
            span_context.trace_id(),
            span_context.span_id(),
            span_context.trace_flags()
        );
        Some(trace_parent)
    } else {
        None
    }
}

#[allow(dead_code)]
struct Traceparent {
    version: u8,
    trace_id: u128, // 16 bytes
    parent_id: u64, // 8 bytes
    flags: u8,
}

impl Traceparent {
    pub fn parse(value: &str) -> Result<Traceparent, String> {
        if value.len() != 55 {
            return Err("traceparent is not of length 55".to_string());
        }
        let segs: Vec<&str> = value.split('-').collect();

        if segs.len() != 4 {
            return Err("traceparent does not have valid number of segments".to_string());
        }

        Ok(Traceparent {
            version: u8::from_str_radix(segs[0], 16).map_err(|err| err.to_string())?,
            trace_id: u128::from_str_radix(segs[1], 16).map_err(|err| err.to_string())?,
            parent_id: u64::from_str_radix(segs[2], 16).map_err(|err| err.to_string())?,
            flags: u8::from_str_radix(segs[3], 16).map_err(|err| err.to_string())?,
        })
    }
}

pub fn trace_parent_to_span(trace_parent: String) -> Result<Context, String> {
    let _ = Traceparent::parse(&trace_parent)?;
    let mut carrier = HashMap::new();
    carrier.insert("traceparent".to_string(), trace_parent);
    let parent_context = global::get_text_map_propagator(|propagator| propagator.extract(&carrier));
    Ok(parent_context)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_trace_parent_parsing() {
        assert!(Traceparent::parse("adsfhasdfs").is_err());
        // right length wrong segments
        assert!(
            Traceparent::parse("00-80e1afed08e019fc1110464cfa66635c-7a085853722dc6d2001").is_err()
        );
        assert!(
            Traceparent::parse("00-80e1afed08e019fc1110464cfa66635c-7a085853722dc6d2-01").is_ok()
        );
    }
}
