---
created: 2026-04-10T00:00:00Z
last_edited: 2026-04-10T00:00:00Z
---

# Cavekit: Observability (OTLP & Jaeger)

## Scope

Optional observability infrastructure for distributed tracing via OpenTelemetry Protocol (OTLP) and Jaeger backend. This is entirely optional; the app works without it, but when enabled, traces are exported to a local Jaeger instance for inspection.

## Requirements

### R1: OTLP/Jaeger Dependency Setup
**Description:** Application can optionally export traces to a Jaeger backend via OTLP protocol.

**Acceptance Criteria:**
- [ ] Four crates added to `Cargo.toml`: `opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp` (with "grpc-tonic" feature), `tracing-opentelemetry`
- [ ] Crate versions are stable and compatible with current Rust version
- [ ] Crates compile without errors or warnings
- [ ] Existing tracing functionality (stdout) is not removed or broken

**Dependencies:** None (infrastructure)

### R2: Conditional Trace Export
**Description:** OTLP export is enabled only when environment variable is set.

**Acceptance Criteria:**
- [ ] Function `init_tracing()` exists in `main.rs` or `src/tracing.rs`
- [ ] If `OTEL_EXPORTER_OTLP_ENDPOINT` environment variable is set, OTLP layer is registered
- [ ] If env var is not set, OTLP layer is not initialized (graceful degradation)
- [ ] Stdout tracing layer is always active regardless of env var
- [ ] No errors or panics occur when env var is missing
- [ ] Example: `OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317` enables export

**Dependencies:** R1 (Dependencies)

### R3: Tracer Provider Initialization
**Description:** OpenTelemetry tracer provider is initialized and configured correctly.

**Acceptance Criteria:**
- [ ] `opentelemetry::global::tracer_provider()` is initialized with `install_batch(Tokio)` (not `install_simple`)
- [ ] Batch exporter uses Tokio runtime for async export
- [ ] Exporter is configured to use gRPC tonic (OTLP over gRPC)
- [ ] Exporter targets the endpoint from `OTEL_EXPORTER_OTLP_ENDPOINT` env var
- [ ] Sampler is set to `AlwaysSampler` (export all traces) for development

**Dependencies:** R2 (Conditional Export)

### R4: Graceful Shutdown
**Description:** Tracer provider is shut down cleanly on application shutdown.

**Acceptance Criteria:**
- [ ] On graceful shutdown signal (SIGTERM/SIGINT), `opentelemetry::global::shutdown_tracer_provider()` is called
- [ ] All pending traces are flushed before shutdown completes
- [ ] Shutdown is non-blocking (does not hang)
- [ ] Application exits cleanly after tracing shutdown

**Dependencies:** R3 (Tracer Provider Initialization)

### R5: Docker Compose for Jaeger
**Description:** Local development includes docker-compose.yml for running Jaeger backend.

**Acceptance Criteria:**
- [ ] `docker-compose.yml` includes `jaeger` service
- [ ] Service uses image: `jaegertracing/all-in-one:latest`
- [ ] OTLP gRPC collector port exposed: 4317 (internal and external)
- [ ] Jaeger UI port exposed: 16686
- [ ] Environment variable `COLLECTOR_OTLP_ENABLED=true` is set
- [ ] Service restarts on failure (restart_policy: unless-stopped)
- [ ] README or comments explain: `docker-compose up jaeger` to start local Jaeger

**Dependencies:** None (infrastructure file)

### R6: Environment Configuration
**Description:** OTLP endpoint is documented as optional configuration.

**Acceptance Criteria:**
- [ ] `.env.example` includes commented-out entry: `# OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317`
- [ ] README or docs mention: "Optional: Enable distributed tracing by setting OTEL_EXPORTER_OTLP_ENDPOINT"
- [ ] Docs explain how to run `docker-compose up jaeger` and then set env var
- [ ] Docs point to Jaeger UI at `http://localhost:16686` after setup

**Dependencies:** R2 (Conditional Export), R5 (Docker Compose)

### R7: Trace Integration with Existing Spans
**Description:** Application's existing trace instrumentation is integrated with OTLP export.

**Acceptance Criteria:**
- [ ] Axum's `TraceLayer` continues to work as before
- [ ] tower-http tracing spans are exported to Jaeger when OTLP is enabled
- [ ] Request/response spans include: method, path, status code, latency
- [ ] Database queries (SQLx) produce child spans (if using tracing-enabled SQLx)
- [ ] Background task spans (polling loop) are exported
- [ ] Manual `tracing::info!()`, `tracing::debug!()` spans are captured
- [ ] Span relationships (parent-child) are preserved in Jaeger UI

**Dependencies:** R3, R4 (Tracer Provider and Shutdown)

### R8: No Breaking Changes
**Description:** Adding OTLP export does not break existing functionality.

**Acceptance Criteria:**
- [ ] `cargo build` succeeds without warnings
- [ ] `cargo test` passes
- [ ] `cargo clippy` passes
- [ ] Application starts and serves requests normally when OTEL_EXPORTER_OTLP_ENDPOINT is not set
- [ ] Stdout logging (tracing-subscriber) continues to work as before
- [ ] No dependencies are downgraded or removed

**Dependencies:** All (overall system)

## Out of Scope

- Metrics export (only tracing/spans, not Prometheus metrics)
- Multiple backend targets (only OTLP/Jaeger)
- Custom samplers or filtering (always-on sampling)
- Trace context propagation to external services (intra-app only)
- W3C Baggage or trace propagation headers
- Jaeger agent mode (using collector container in all-in-one)
- Production deployment configuration (TLS, authentication, sampling)
- Trace storage backend choices (defer to Jaeger defaults)
- Alerting on trace anomalies

## Implementation Notes

### Typical init_tracing() Structure (Rust pseudocode)
```rust
pub fn init_tracing() -> anyhow::Result<()> {
    let stdout_layer = tracing_subscriber::fmt::layer();
    
    let subscriber = if let Ok(endpoint) = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
        let tracer = opentelemetry_otlp::new_pipeline()
            .tracing()
            .with_exporter(
                opentelemetry_otlp::new_exporter()
                    .tonic()
                    .with_endpoint(endpoint)
            )
            .install_batch(opentelemetry_sdk::runtime::Tokio)?;
        
        let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);
        tracing_subscriber::registry()
            .with(stdout_layer)
            .with(otel_layer)
            .init();
    } else {
        tracing_subscriber::registry()
            .with(stdout_layer)
            .init();
    };
    
    Ok(())
}

// On shutdown:
opentelemetry::global::shutdown_tracer_provider();
```

### Docker Compose Entry (Minimal Example)
```yaml
version: '3.8'
services:
  jaeger:
    image: jaegertracing/all-in-one:latest
    ports:
      - "4317:4317"  # OTLP gRPC
      - "16686:16686"  # Web UI
    environment:
      - COLLECTOR_OTLP_ENABLED=true
    restart: unless-stopped
```

## Source Traceability

### Greenfield Status: New Infrastructure (Task 0010)
This cavekit describes optional observability infrastructure not yet implemented.

### Related Task
- **Task 0010** — OTLP/Jaeger observability — full implementation in one task

### Source Files (To Be Created/Modified)
- `src/tracing.rs` — new module with `init_tracing()` function
- `src/main.rs` — call `init_tracing()` on startup, call shutdown on graceful shutdown
- `docker-compose.yml` — new file with jaeger service
- `.env.example` — add OTEL_EXPORTER_OTLP_ENDPOINT entry
- `Cargo.toml` — add 4 new crates
- `README.md` — update development section with Jaeger setup

### Testing Checklist
- [ ] Run `cargo build` and `cargo clippy` — no errors
- [ ] Run `cargo test` — all tests pass
- [ ] Without OTEL_EXPORTER_OTLP_ENDPOINT set: app runs normally, logs to stdout
- [ ] With OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317: app runs, exports traces
- [ ] Run `docker-compose up jaeger` in separate terminal
- [ ] Visit http://localhost:16686 and verify traces appear for HTTP requests
- [ ] Verify trace parent-child relationships are preserved

## Cross-References
- Depends on: None (infrastructure-only, independent of business logic)
- Related to: All domains (may trace any module)
- Optional enhancement for: **cavekit-scoring.md** (polling task traces), **cavekit-predictions.md** (form submission traces)
