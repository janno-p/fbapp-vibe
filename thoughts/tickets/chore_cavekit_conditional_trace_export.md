---
type: chore
priority: high
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, observability, tracing, env-config]
keywords: [init_tracing, OTEL_EXPORTER_OTLP_ENDPOINT, tracing subscriber, conditional export]
patterns: [feature flag via env var, graceful degradation, additive subscriber layers, optional integration]
---

# CHORE-OBS-02: Conditional OTLP trace export

## Description
Enable OTLP trace export only when the OTLP endpoint environment variable is present.

## Context
The observability stack must remain optional so developers without Jaeger can run the app normally.

## Requirements
- Provide an `init_tracing()` function in `main.rs` or `src/tracing.rs`.
- Register the OTLP layer only when `OTEL_EXPORTER_OTLP_ENDPOINT` is set.
- Skip OTLP initialization when the variable is missing.
- Keep the stdout tracing layer active in all cases.
- Avoid panics or errors when the variable is absent.
- Support `OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317` as the enablement path.

### Functional Requirements
- Toggle OTLP export purely through environment configuration.
- Preserve the existing stdout tracing behavior.

### Non-Functional Requirements
- Startup must degrade gracefully when OTLP is disabled.
- No hard dependency on Jaeger for local development.

## Current State
There is no conditional OTLP wiring yet.

## Desired State
The app enables trace export only when configured, otherwise it behaves exactly as before.

## Research Context

### Keywords to Search
- `init_tracing` - tracing bootstrap entry point
- `OTEL_EXPORTER_OTLP_ENDPOINT` - enablement flag
- `tracing_subscriber` - subscriber composition
- `stdout layer` - existing always-on tracing output

### Patterns to Investigate
- feature flag via env var - how optional runtime behavior is controlled
- graceful degradation - how startup handles missing config
- additive subscriber layers - how to compose stdout and OTLP layers

### Key Decisions Made
- OTLP export is opt-in only.
- Stdout tracing remains on regardless of configuration.

## Success Criteria
The ticket is complete when OTLP export can be toggled with one environment variable and the app still starts without it.

### Automated Verification
- [ ] Test or integration check covers enabled OTLP initialization.
- [ ] Test or integration check covers missing-env startup.

### Manual Verification
- [ ] App starts cleanly with no OTLP environment variable set.
- [ ] App registers OTLP export when `OTEL_EXPORTER_OTLP_ENDPOINT` is set.

## Related Information
- Source doc: `context/kits/cavekit-observability.md`
- Requirement: `R2`

## Notes
Do not include shutdown handling or Jaeger container config here.
