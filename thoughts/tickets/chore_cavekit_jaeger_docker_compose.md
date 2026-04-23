---
type: chore
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, observability, docker, jaeger]
keywords: [docker-compose, jaeger, 4317, 16686, COLLECTOR_OTLP_ENABLED]
patterns: [local dev infrastructure, container service config, exposed ports, restart policy]
---

# CHORE-OBS-05: Jaeger docker compose support

## Description
Add local docker-compose support for running a Jaeger all-in-one container.

## Context
Developers need a simple local backend to inspect exported traces without external infrastructure.

## Requirements
- Add a `jaeger` service to `docker-compose.yml`.
- Use the `jaegertracing/all-in-one:latest` image.
- Expose OTLP gRPC port `4317`.
- Expose Jaeger UI port `16686`.
- Set `COLLECTOR_OTLP_ENABLED=true`.
- Use a restart policy such as `unless-stopped`.
- Document how to start Jaeger with `docker-compose up jaeger`.

### Functional Requirements
- Provide a local backend for trace ingestion and inspection.
- Make the Jaeger UI reachable from the developer machine.

### Non-Functional Requirements
- The container should be easy to start and resilient to restarts.
- Compose config should stay minimal and dev-focused.

## Current State
The compose file does not yet include Jaeger.

## Desired State
Developers can start Jaeger locally and inspect traces through the UI.

## Research Context

### Keywords to Search
- `docker-compose.yml` - local service definition
- `jaegertracing/all-in-one:latest` - container image
- `4317` - OTLP gRPC collector port
- `16686` - Jaeger UI port
- `COLLECTOR_OTLP_ENABLED=true` - collector config

### Patterns to Investigate
- local dev infrastructure - how compose services are documented
- container service config - restart and port exposure patterns
- exposed ports - how local tracing services are reached

### Key Decisions Made
- Use Jaeger all-in-one mode.
- Keep the configuration local-development only.

## Success Criteria
The ticket is complete when Jaeger can be started locally and the UI is reachable on port 16686.

### Automated Verification
- [ ] `docker compose config` validates the compose file.

### Manual Verification
- [ ] `docker compose up jaeger` starts the service.
- [ ] `http://localhost:16686` loads the Jaeger UI.

## Related Information
- Source doc: `context/kits/cavekit-observability.md`
- Requirement: `R5`

## Notes
Keep any README updates limited to this compose workflow.
