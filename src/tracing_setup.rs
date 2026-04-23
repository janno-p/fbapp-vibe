/// Tracing initialisation with optional OTLP/Jaeger export.
/// Implements cavekit-observability.md R2, R3, R4.
///
/// If `OTEL_EXPORTER_OTLP_ENDPOINT` is set, a batch gRPC exporter is registered.
/// Stdout tracing is always active regardless of that env var.
use tracing_subscriber::{EnvFilter, Registry, layer::SubscriberExt, util::SubscriberInitExt};

type BoxedLayer = Box<dyn tracing_subscriber::Layer<Registry> + Send + Sync + 'static>;

/// Initialise tracing. Returns `true` if OTLP export was enabled.
pub fn init_tracing() -> bool {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "fbapp_vibe=debug,tower_http=debug,sqlx=warn".into());
    let fmt_layer = tracing_subscriber::fmt::layer();

    let (otel_layer, enabled): (Option<BoxedLayer>, bool) =
        match std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
            Ok(endpoint) => match build_otlp_layer(&endpoint) {
                Ok(layer) => (Some(layer), true),
                Err(e) => {
                    eprintln!("[tracing] OTLP setup failed, falling back to stdout: {e:#}");
                    (None, false)
                }
            },
            Err(_) => (None, false),
        };

    tracing_subscriber::registry()
        .with(otel_layer) // Option<BoxedLayer> — no-op when None
        .with(env_filter)
        .with(fmt_layer)
        .init();

    if enabled {
        tracing::info!("OTLP tracing enabled");
    }

    enabled
}

fn build_otlp_layer(endpoint: &str) -> anyhow::Result<BoxedLayer> {
    use opentelemetry::KeyValue;
    use opentelemetry::sdk::Resource;
    use opentelemetry::sdk::trace::config;
    use opentelemetry_otlp::WithExportConfig;

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(
            opentelemetry_otlp::new_exporter()
                .tonic()
                .with_endpoint(endpoint),
        )
        .with_trace_config(
            config()
                .with_sampler(opentelemetry::sdk::trace::Sampler::AlwaysOn)
                .with_resource(Resource::new(vec![KeyValue::new(
                    "service.name",
                    "fbapp-vibe",
                )])),
        )
        .install_batch(opentelemetry::runtime::Tokio)?;

    let layer: BoxedLayer = Box::new(tracing_opentelemetry::layer().with_tracer(tracer));
    Ok(layer)
}

/// Flush and shut down the global tracer provider on graceful shutdown.
/// Implements cavekit-observability.md R4.
pub fn shutdown_tracing() {
    opentelemetry::global::shutdown_tracer_provider();
}
