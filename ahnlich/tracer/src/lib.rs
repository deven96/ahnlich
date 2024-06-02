use opentelemetry::{global, KeyValue};
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::{
    trace::{self, Sampler},
    Resource,
};
use tracing::subscriber::set_global_default;
use tracing_subscriber::{layer::SubscriberExt, EnvFilter, Registry};

pub fn init_tracing(service_name: &'static str, log_level: Option<&str>, otel_url: &str) {
    let env_filter = EnvFilter::new(log_level.unwrap_or_else(|| "info"));
    //let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

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
}

pub fn shutdown_tracing() {
    global::shutdown_tracer_provider();
}

#[cfg(test)]
mod tests {}
