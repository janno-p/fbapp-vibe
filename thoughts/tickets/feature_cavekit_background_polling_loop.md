---
type: feature
priority: medium
created: 2026-04-23T00:00:00Z
status: created
tags: [cavekit, polling, infrastructure, scoring]
keywords: [background polling, tokio::spawn, polling interval, graceful shutdown, warn logging, debug logging]
patterns: [background worker, retry on failure, dynamic interval, shutdown-aware loop, telemetry logging]
---

# FEATURE-SCORING-01: Background polling loop for cavekit scoring

## Summary

Run a long-lived background task that periodically polls for tournament results and drives the scoring pipeline.

## Acceptance Criteria

- [ ] The polling task is spawned from `main.rs` with `tokio::spawn`.
- [ ] The loop uses a 30 second interval when the tournament is active and has live matches.
- [ ] The loop uses a 120 second interval when the tournament is inactive or has no live matches.
- [ ] Each polling cycle is logged at debug level.
- [ ] API or database failures are logged at warn level and do not crash the task.
- [ ] The loop respects shutdown signals and exits cleanly.

## Implementation Context

### Relevant files

- `src/main.rs` - spawn the polling task.
- `src/polling/mod.rs` - loop orchestration and interval selection.
- `src/state.rs` - shared app state passed into the task.
- `src/config.rs` - interval configuration if it needs to be exposed.

### Tests

- Unit test the interval selection logic.
- Integration test that a failed cycle is logged and the loop continues.

### Implementation notes

- The task should be event-loop style, not a one-shot job.
- Keep shutdown handling explicit so deployment restarts do not leave a stray worker.

## Research Context

### Keywords to Search

- `tokio::spawn` - task entry point.
- `polling interval` - dynamic timing logic.
- `graceful shutdown` - stop condition.
- `warn level` - failure logging behavior.

### Patterns to Investigate

- background worker - how long-lived loops are structured.
- retry on failure - how transient failures are handled.
- dynamic interval - how polling cadence is adjusted.

### Key Decisions Made

- Polling is server-side only.
- Failures are non-fatal and retried on the next cycle.

## Success Criteria

### Automated Verification

- [ ] `cargo test` covers interval and shutdown behavior.
- [ ] `cargo clippy -- -D warnings` passes for the polling module.

### Manual Verification

- [ ] Server starts the polling worker automatically.
- [ ] Logs show repeated cycles without crashes.

## Related Information

- Source requirement: `context/kits/cavekit-scoring.md` R1.
- Depends on the active tournament state existing in the database.

## Notes

- This ticket intentionally excludes result ingestion details; those belong in a separate ticket.
