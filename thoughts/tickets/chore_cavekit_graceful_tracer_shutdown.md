---
type: chore
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, observability, tracing, shutdown]
keywords: [shutdown_tracer_provider, SIGTERM, SIGINT, flush traces, graceful shutdown]
patterns: [signal handling, non-blocking shutdown, buffered export flush, cleanup hook]
---

# CHORE-OBS-04: Graceful tracer shutdown

## Description
Shut down the tracer provider cleanly so pending traces are flushed during application exit.

## Context
Without explicit shutdown, buffered spans can be lost when the process exits.

## Requirements
- Call `opentelemetry::global::shutdown_tracer_provider()` on graceful shutdown.
- Handle SIGTERM and SIGINT shutdown paths.
- Flush pending traces before exit completes.
- Avoid blocking or hanging during shutdown.
- Exit cleanly after tracing shutdown completes.

### Functional Requirements
- Ensure trace buffers are drained on shutdown.
- Preserve clean process termination.

### Non-Functional Requirements
- Shutdown must remain fast and non-blocking.
- Cleanup should not destabilize normal app exit.

## Current State
No explicit tracer shutdown path exists.

## Desired State
The app flushes and stops OpenTelemetry tracing cleanly when it shuts down.

## Research Context

### Keywords to Search
- `shutdown_tracer_provider` - cleanup entry point
- `SIGTERM` - graceful shutdown signal
- `SIGINT` - interrupt shutdown signal
- `flush traces` - buffered export handling

### Patterns to Investigate
- signal handling - how shutdown hooks are wired
- non-blocking shutdown - how to avoid hangs
- buffered export flush - what must complete before exit

### Key Decisions Made
- Tracer shutdown is part of graceful process teardown.
- Pending spans should be flushed before the process exits.

## Success Criteria
The ticket is complete when graceful shutdown flushes traces and the app still exits promptly.

### Automated Verification
- [ ] Shutdown path test or integration check covers tracer cleanup.

### Manual Verification
- [ ] App exits cleanly on SIGTERM or SIGINT.
- [ ] Pending traces are visible in Jaeger after shutdown.

## Related Information
- Source doc: `context/kits/cavekit-observability.md`
- Requirement: `R4`

## Notes
Do not mix in startup initialization logic here.
