# ADR-0017: OTLP Trace Export with Jaeger for Local Observability 🔭

## Status

✅ Accepted

## Extends

[ADR-0010](0010-observability-with-tracing.md) — Use tracing for Observability

## Date

2026-04-06

## Context

ADR-0010 established `tracing` + `tracing-subscriber` with stdout output for local development. It explicitly anticipated OpenTelemetry export as a future step:

> *"Switching to JSON output or adding OpenTelemetry export requires only changes to the subscriber initialisation — no application code changes."*

The motivation is to replace purely stdout-based development observability with a UI that shows:
- Distributed traces with span timings and attributes
- Request waterfalls across middleware, handlers, and DB queries
- Structured log events correlated to traces

The context for this decision is a comparison against .NET Aspire for local orchestration. Aspire was rejected (ADR not written) because it requires the .NET SDK and its non-.NET support is second-class. The Aspire dashboard's observability UI is the one genuinely valuable feature — this ADR captures the equivalent without the .NET dependency.

### Backend options evaluated

| Option | Pros | Cons |
|---|---|---|
| **Jaeger all-in-one** | Single container, zero config, native OTLP since v1.35, good trace UI | Traces only; no metrics or log aggregation |
| Grafana + Tempo + Loki + Prometheus | Full observability stack, production-like | 4–5 containers, non-trivial config, overkill for a single-service app |
| Zipkin | Mature, simple | Older OTLP support, inferior UI to Jaeger |
| stdout JSON + jq | No extra tooling | No correlation, no UI, tedious |

### Why Jaeger all-in-one

This application is a single Rust service with one PostgreSQL dependency. The primary local development need is **trace correlation** — seeing which DB query was slow, which middleware layer added latency, which handler errored. Jaeger all-in-one delivers exactly this in a single `docker compose up` with no configuration files.

Metrics (Prometheus) and log aggregation (Loki) are not needed locally: `RUST_LOG` stdout output is sufficient for log browsing during development, and application-level metrics are not yet instrumented. These can be added in a future ADR if the need arises.

## Decision

We will export OpenTelemetry traces from the Rust application to **Jaeger all-in-one** via OTLP/gRPC, adding:

### Rust crates

```toml
opentelemetry          = "0.27"
opentelemetry_sdk      = { version = "0.27", features = ["rt-tokio"] }
opentelemetry-otlp     = { version = "0.27", features = ["grpc-tonic"] }
tracing-opentelemetry  = "0.28"
tonic                  = "0.12"   # required by grpc-tonic feature
```

### Subscriber initialisation

OTLP export is **opt-in via environment variable**. When `OTEL_EXPORTER_OTLP_ENDPOINT` is not set, the application behaves exactly as before (stdout only). This keeps `cargo run` without docker working for quick iterations.

```rust
fn init_tracing() -> Option<opentelemetry_sdk::trace::Tracer> {
    let env_filter = EnvFilter::try_from_default_env()
        .unwrap_or_else(|_| "fbapp_vibe=debug,tower_http=debug,sqlx=warn".into());

    let fmt_layer = tracing_subscriber::fmt::layer();

    let registry = tracing_subscriber::registry()
        .with(env_filter)
        .with(fmt_layer);

    if let Ok(endpoint) = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(endpoint),
            )
            .with_trace_config(
                opentelemetry_sdk::trace::Config::default()
                    .with_resource(opentelemetry_sdk::Resource::new(vec![
                        opentelemetry::KeyValue::new("service.name", "fbapp-vibe"),
                    ])),
            )
            .install_batch(opentelemetry_sdk::runtime::Tokio)
            .expect("failed to initialise OTLP tracer");

        registry
            .with(tracing_opentelemetry::layer().with_tracer(tracer.clone()))
            .init();

        Some(tracer)
    } else {
        registry.init();
        None
    }
}
```

On graceful shutdown, flush the tracer pipeline:

```rust
// in main.rs, before process exit
if let Some(_tracer) = tracer {
    opentelemetry::global::shutdown_tracer_provider();
}
```

### Docker Compose addition

```yaml
jaeger:
  image: jaegertracing/all-in-one:latest
  ports:
    - "4317:4317"   # OTLP gRPC receiver
    - "16686:16686" # Jaeger UI
  environment:
    COLLECTOR_OTLP_ENABLED: "true"
```

### Environment variables

```bash
# .env (local dev with docker compose)
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317

# .env.example — document but leave blank
OTEL_EXPORTER_OTLP_ENDPOINT=   # optional; set to http://localhost:4317 when running Jaeger
```

### Makefile

No changes needed. `make dev` continues to work with or without Jaeger running.

## Rationale

- 🪶 **Zero-config Jaeger**: `jaegertracing/all-in-one` requires no config files; OTLP is enabled via a single env var. The full trace store, query engine, and UI are in one container.
- 🔌 **Additive change**: existing stdout logging is untouched. OTLP is an additional layer, not a replacement. Developers without docker still get full stdout output.
- 🦀 **`tracing-opentelemetry` bridge**: this crate converts `tracing` spans to OpenTelemetry spans automatically. No application code changes are needed — existing `#[tracing::instrument]` annotations and `TraceLayer` instrumentation are automatically exported.
- ✂️ **Deferred complexity**: metrics (Prometheus) and log aggregation (Loki/Grafana) are explicitly out of scope. They can be added incrementally if needed without revisiting this decision.

## Trade-offs and Risks ⚠️

- ⚠️ **Traces only**: Jaeger does not aggregate metrics or logs. Structured logs remain in stdout. This is acceptable for the current scale and team size.
- ⚠️ **`tonic` compile time**: the `grpc-tonic` feature pulls in `tonic` which increases compile time. If this becomes painful, the `http-proto` feature (using `reqwest`) is a drop-in alternative.
- ⚠️ **In-memory storage**: `all-in-one` stores traces in memory; they are lost on container restart. For local dev this is fine; a production deployment would use a persistent backend (Jaeger with Cassandra/Elasticsearch, or Grafana Tempo).
- ⚠️ **`latest` image tag**: pinning to `latest` means the Jaeger UI may change across pulls. Pin to a specific version (e.g. `jaegertracing/all-in-one:1.57`) once the setup is stable.

## Consequences

- 🔭 Traces from all `#[tracing::instrument]` functions, `TraceLayer` HTTP middleware, and SQLx queries are visible in the Jaeger UI at `http://localhost:16686` when running with docker compose.
- 🔌 `OTEL_EXPORTER_OTLP_ENDPOINT` absent → stdout only (no behaviour change for existing workflows).
- 📦 Four new crate dependencies: `opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp`, `tracing-opentelemetry`.
- 🛑 Graceful shutdown must call `opentelemetry::global::shutdown_tracer_provider()` to flush buffered spans before exit.
- 🚀 Production deployment (ADR-0012) will need `OTEL_EXPORTER_OTLP_ENDPOINT` pointed at a real OTLP backend or left unset to fall back to stdout JSON.
