---
type: chore
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, observability, docs, env]
keywords: [OTEL_EXPORTER_OTLP_ENDPOINT, .env.example, optional configuration, Jaeger UI]
patterns: [configuration docs, commented env example, developer onboarding, optional feature docs]
---

# CHORE-OBS-06: OTLP environment configuration docs

## Description
Document the optional environment configuration needed to enable OTLP trace export.

## Context
The tracing feature should be discoverable without forcing every developer to enable it.

## Requirements
- Add a commented-out `OTEL_EXPORTER_OTLP_ENDPOINT` entry to `.env.example`.
- Document that OTLP tracing is optional.
- Explain how to run `docker-compose up jaeger`.
- Point developers to the Jaeger UI at `http://localhost:16686`.

### Functional Requirements
- Make the enablement path easy to find.
- Keep the documentation aligned with the compose setup.

### Non-Functional Requirements
- Documentation should not imply that Jaeger is required.
- Setup instructions should be short and unambiguous.

## Current State
The optional OTLP configuration is not documented yet.

## Desired State
Developers can enable tracing using the documented env var and local Jaeger setup.

## Research Context

### Keywords to Search
- `.env.example` - environment template file
- `OTEL_EXPORTER_OTLP_ENDPOINT` - optional tracing endpoint
- `docker-compose up jaeger` - local setup instruction
- `http://localhost:16686` - Jaeger UI URL

### Patterns to Investigate
- configuration docs - how optional env vars are documented
- commented env example - how to show defaults without enabling them
- developer onboarding - where local setup steps live

### Key Decisions Made
- Keep OTLP opt-in and documented as such.
- Reference the local Jaeger UI directly in docs.

## Success Criteria
The ticket is complete when the optional tracing setup is documented clearly in the repo.

### Automated Verification
- [ ] `.env.example` contains the commented OTLP endpoint.
- [ ] Documentation build or lint checks still pass, if applicable.

### Manual Verification
- [ ] A developer can follow the docs to enable tracing locally.

## Related Information
- Source doc: `context/kits/cavekit-observability.md`
- Requirement: `R6`

## Notes
Do not expand this into a broader developer docs cleanup.
