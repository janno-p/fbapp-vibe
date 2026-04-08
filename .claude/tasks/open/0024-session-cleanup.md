---
id: 0024
title: Session table cleanup job
status: open
phase: MVP
type: chore
adrs: []
refs: []
created: 2026-04-07
started: ~
completed: ~
---

## Goal

`tower_sessions` stores sessions in the `tower_sessions` PostgreSQL table and never cleans up expired rows. Over time this table will grow unboundedly. A periodic cleanup removes rows whose expiry has passed, keeping the table small and index efficient.

## Acceptance Criteria

- [ ] Expired sessions are deleted from the `tower_sessions` table on a schedule
- [ ] Cleanup runs in the background without blocking request handling
- [ ] The cleanup interval is reasonable (e.g. once per hour)
- [ ] Cleanup is logged at `debug` level with the number of rows deleted

## Context for Claude 🤖

### Relevant files

- `src/main.rs` — spawn background task alongside the polling task
- `src/polling/mod.rs` — reference for how background tasks are spawned
- `tower_sessions_sqlx_store` crate — check if `PostgresStore` provides a `delete_expired()` method or equivalent; if not, use a raw SQL query

### ADR constraints

- Background tasks follow the `tokio::spawn` pattern already used for polling

### Tests

- No tests — trivial periodic DELETE

### Implementation notes

- Check `tower_sessions_sqlx_store::PostgresStore` docs: it may have a `continuously_delete_expired(interval)` method that handles this automatically. If so, call it instead of writing a custom task.
- If implementing manually: `DELETE FROM tower_sessions WHERE expiry_date < NOW()` (exact column name depends on crate version — check the migration or table schema).
- The task should loop with `tokio::time::sleep(Duration::from_secs(3600))` — no config knob needed unless cleanup starts taking too long.
- Cleanup failure should log a warning and continue, not crash.

## Outcome

_Fill in after completion._
