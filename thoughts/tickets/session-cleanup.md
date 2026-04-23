---
title: Session table cleanup job
source: .claude/tasks/done/0024-session-cleanup.md
source_id: 0024
source_status: open
source_title: Session table cleanup job
status: open
phase: MVP
type: chore
adrs: []
refs: []
created: 2026-04-07
started: ~
completed: ~
---

## Summary

`tower_sessions` stores sessions in the `tower_sessions` PostgreSQL table and never cleans up expired rows. Over time this table will grow unboundedly. A periodic cleanup removes rows whose expiry has passed, keeping the table small and index efficient.

## Acceptance Criteria

- [ ] Expired sessions are deleted from the `tower_sessions` table on a schedule
- [ ] Cleanup runs in the background without blocking request handling
- [ ] The cleanup interval is reasonable (e.g. once per hour)
- [ ] Cleanup is logged at `debug` level with the number of rows deleted

## Implementation Context

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

Added `src/session_cleanup.rs` as a non-route module with a `run(pool: PgPool)` async function. It sleeps 1 hour then executes `DELETE FROM tower_sessions WHERE expiry_date < NOW()` via a raw sqlx query, logs rows deleted at `debug` level, and logs warnings on failure before continuing the loop. The pool is cloned before `AppState::new()` in `main.rs` so both the state and the cleanup task share the same `PgPool`. Module declared in `lib.rs` and spawned alongside the polling task.
