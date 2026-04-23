---
type: chore
priority: high
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, observability, tracing, runtime]
keywords: [tracer provider, install_batch, Tokio, grpc tonic, AlwaysSampler]
patterns: [batch export, async runtime integration, tracer provider bootstrap, development sampling]
---

# CHORE-OBS-03: Tracer provider initialization

## Description
Initialize the OpenTelemetry tracer provider with the correct batch export runtime and development sampling behavior.

## Context
This ticket covers the core runtime wiring that connects the OTLP dependency setup to actual trace export.

## Requirements
- Initialize the global tracer provider with `install_batch(Tokio)`.
- Do not use `install_simple`.
- Configure OTLP export over gRPC tonic.
- Use the endpoint from `OTEL_EXPORTER_OTLP_ENDPOINT`.
- Set the sampler to `AlwaysSampler` for development.

### Functional Requirements
- Create a working tracer provider when OTLP export is enabled.
- Export traces asynchronously through the Tokio runtime.

### Non-Functional Requirements
- Batch export must not block request handling.
- The provider configuration should be suitable for local development.

## Current State
Tracer provider initialization is not implemented.

## Desired State
The app creates a correctly configured OpenTelemetry tracer provider when OTLP export is enabled.

## Research Context

### Keywords to Search
- `tracer provider` - OpenTelemetry runtime entry point
- `install_batch` - required initialization style
- `Tokio` - async runtime for batch export
- `AlwaysSampler` - development sampling policy
- `grpc tonic` - exporter transport

### Patterns to Investigate
- batch export - runtime-safe trace flushing
- async runtime integration - when initialization must occur
- tracer provider bootstrap - how the tracing pipeline is built

### Key Decisions Made
- Use batch export on Tokio.
- Export all traces in development.

## Success Criteria
The ticket is complete when the tracer provider is initialized with batch export and the configured endpoint is used.

### Automated Verification
- [ ] Test confirms `install_batch(Tokio)`-style initialization path.
- [ ] Test confirms the endpoint is read from the environment.

### Manual Verification
- [ ] Traces export through OTLP when enabled.

## Related Information
- Source doc: `context/kits/cavekit-observability.md`
- Requirement: `R3`

## Notes
Keep shutdown cleanup separate so initialization stays focused.
