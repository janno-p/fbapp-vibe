---
title: OTLP trace export with Jaeger
source: .claude/tasks/open/0010-otlp-jaeger-observability.md
source_id: 0010
source_status: open
source_title: OTLP trace export with Jaeger
status: open
phase: Phase3
type: chore
adrs: [0017]
refs: []
created: 2026-04-06
started: ~
completed: ~
---

## Summary

Wire up OpenTelemetry OTLP trace export so that all existing `tracing` instrumentation (HTTP middleware, handler spans, SQLx queries) is automatically visible in a local Jaeger UI. The change must be fully opt-in — the app must continue to work identically without any environment changes for developers not running Jaeger.

## Acceptance Criteria

- [ ] Four crates added to `Cargo.toml`: `opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp` (grpc-tonic), `tracing-opentelemetry`
- [ ] `init_tracing()` in `main.rs` conditionally registers the OTLP layer when `OTEL_EXPORTER_OTLP_ENDPOINT` is set; stdout layer is always active
- [ ] `opentelemetry::global::shutdown_tracer_provider()` is called on graceful shutdown
- [ ] `docker-compose.yml` includes a `jaeger` service (see ADR-0017 for config)
- [ ] `OTEL_EXPORTER_OTLP_ENDPOINT` documented in `.env.example` as optional
- [ ] `cargo run` without `OTEL_EXPORTER_OTLP_ENDPOINT` set works as before — no panics, no connection errors
- [ ] With docker compose running, traces from HTTP requests appear in Jaeger UI at `http://localhost:16686`
- [ ] `cargo clippy` and `cargo test` pass

## Implementation Context

### Relevant files

- `src/main.rs` — update `init_tracing()` function
- `Cargo.toml` — add four crates
- `docker-compose.yml` — add `jaeger` service
- `.env.example` — document new optional env var

### ADR constraints

- **ADR-0017**: Follow the exact subscriber initialisation pattern from the ADR; OTLP is opt-in via env var; use `install_batch(Tokio)` not `install_simple`
- **ADR-0010**: Stdout layer must remain active at all times; OTLP is additive

### Tests

- No automated tests for this task — OTLP export is infrastructure wiring with no business logic. The acceptance criteria "traces appear in Jaeger UI" is verified manually.
- Existing `cargo test` must continue to pass after the crate additions.

### Implementation notes

- The `init_tracing()` function currently returns `()`; change it to return `Option<opentelemetry_sdk::trace::Tracer>` so main can call shutdown on it
- `install_batch` requires the `opentelemetry_sdk` `rt-tokio` feature and must be called after the Tokio runtime is started — `init_tracing()` is called inside `main` after `#[tokio::main]`, so this is fine
- SQLx does not emit OpenTelemetry spans natively, but it does emit `tracing` spans — these will appear in Jaeger automatically via the `tracing-opentelemetry` bridge
- The `tonic` crate is a transitive dependency of `opentelemetry-otlp`'s `grpc-tonic` feature; do not add it explicitly unless a version conflict requires pinning

### Jaeger docker-compose snippet (from ADR-0017)

```yaml
jaeger:
  image: jaegertracing/all-in-one:latest
  ports:
    - "4317:4317"   # OTLP gRPC
    - "16686:16686" # UI
  environment:
    COLLECTOR_OTLP_ENABLED: "true"
```

## Outcome

> Fill this section in after implementation, before moving it to the done archive.

Follow-up tasks: _none_
