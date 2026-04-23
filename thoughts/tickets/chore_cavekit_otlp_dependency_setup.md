---
type: chore
priority: high
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, observability, tracing, dependencies]
keywords: [opentelemetry, opentelemetry_sdk, opentelemetry-otlp, tracing-opentelemetry, Cargo.toml]
patterns: [dependency setup, crate version compatibility, tracing stack wiring, build validation]
---

# CHORE-OBS-01: OTLP/Jaeger dependency setup

## Description
Add the OpenTelemetry crates needed to support optional OTLP trace export to Jaeger.

## Context
This is the foundation for all later observability work and must not break the existing stdout tracing setup.

## Requirements
- Add `opentelemetry` to `Cargo.toml`.
- Add `opentelemetry_sdk` to `Cargo.toml`.
- Add `opentelemetry-otlp` with the `grpc-tonic` feature.
- Add `tracing-opentelemetry` to `Cargo.toml`.

### Functional Requirements
- Keep the existing tracing stack intact.
- Make the dependency set compatible with the current Rust toolchain.

### Non-Functional Requirements
- No compile errors or warnings from the added crates.
- No regression in existing stdout logging/tracing behavior.

## Current State
The observability kit is still greenfield and these crates are not yet added.

## Desired State
The project can compile with the OpenTelemetry tracing dependencies available for later wiring.

## Research Context

### Keywords to Search
- `opentelemetry` - core tracing API crate
- `opentelemetry_sdk` - tracer provider and runtime support
- `opentelemetry-otlp` - OTLP exporter crate
- `tracing-opentelemetry` - bridge between tracing and OpenTelemetry
- `Cargo.toml` - dependency declaration file

### Patterns to Investigate
- dependency setup - how existing crates are pinned and grouped
- tracing stack wiring - how stdout tracing is configured today
- build validation - how dependency additions are verified

### Key Decisions Made
- Use the gRPC tonic exporter path.
- Treat the OTLP stack as additive to existing stdout tracing.

## Success Criteria
The ticket is complete when the project builds with the new tracing dependencies and stdout tracing still works.

### Automated Verification
- [ ] `cargo build` succeeds after dependency changes.
- [ ] `cargo clippy` passes without new warnings.

### Manual Verification
- [ ] Existing startup behavior still produces stdout traces.

## Related Information
- Source doc: `context/kits/cavekit-observability.md`
- Requirement: `R1`

## Notes
Keep this ticket limited to dependency declarations; runtime initialization belongs in a separate ticket.
