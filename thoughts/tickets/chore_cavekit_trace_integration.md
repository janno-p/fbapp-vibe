---
type: chore
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, observability, tracing, instrumentation]
keywords: [TraceLayer, tower-http, sqlx spans, background task spans, tracing::info]
patterns: [span propagation, child spans, request tracing, instrumentation bridge]
---

# CHORE-OBS-07: Trace integration with existing spans

## Description
Connect the existing tracing instrumentation so HTTP, database, and background spans are exported to Jaeger when OTLP is enabled.

## Context
This ticket verifies that the observability stack captures real application activity, not just startup wiring.

## Requirements
- Keep Axum's `TraceLayer` working as before.
- Export tower-http request and response spans to Jaeger.
- Include request method, path, status code, and latency in spans.
- Export SQLx child spans when tracing is available.
- Export background task spans such as the polling loop.
- Capture manual `tracing::info!()` and `tracing::debug!()` spans.
- Preserve parent-child span relationships in Jaeger.

### Functional Requirements
- Surface the existing app instrumentation in Jaeger.
- Preserve the current shape of request tracing.

### Non-Functional Requirements
- Span relationships must remain intact.
- Instrumentation changes must not break existing logs or request handling.

## Current State
Tracing exists locally, but it is not yet bridged into OTLP export.

## Desired State
Existing spans show up in Jaeger with meaningful parent-child relationships.

## Research Context

### Keywords to Search
- `TraceLayer` - HTTP tracing middleware
- `tower-http` - request span source
- `sqlx spans` - database tracing integration
- `background task spans` - polling or async work
- `tracing::info!` - manual span emission

### Patterns to Investigate
- span propagation - parent-child relationships across layers
- request tracing - standard HTTP instrumentation fields
- instrumentation bridge - how tracing emits to OpenTelemetry

### Key Decisions Made
- Existing instrumentation should be reused, not replaced.
- Parent-child relationships are required in Jaeger.

## Success Criteria
The ticket is complete when real app spans are visible in Jaeger and the relationships are correct.

### Automated Verification
- [ ] Test covers request span export fields.
- [ ] Test covers background span export, if feasible.

### Manual Verification
- [ ] HTTP requests appear in Jaeger with method, path, status, and latency.
- [ ] Span nesting looks correct in the Jaeger UI.

## Related Information
- Source doc: `context/kits/cavekit-observability.md`
- Requirement: `R7`

## Notes
Do not bundle shutdown or dependency updates into this ticket.
