# Plan: Auth Tier 2 — Session Cleanup

## Source Kits
- cavekit-auth.md: R5

## Implementation Sequence

### T-006: Verify Session Cleanup Background Task
**Cavekit Requirement:** cavekit-auth/R5
**Acceptance Criteria Mapped:**
- Background task runs on a defined schedule (hourly or per configuration)
- Background task deletes all rows from `tower_sessions` where `expiry_date <= now()`
- Cleanup task logs the number of sessions deleted at info level
- Cleanup task continues even if no sessions exist to clean
- Cleanup task restarts automatically if it crashes (supervisor responsibility, not this cavekit)

**blockedBy:** T-003
**Effort:** M
**Description:**
1. Read `src/session_cleanup.rs` — verify background task exists
2. Verify task is spawned from `main.rs` (likely via `tokio::spawn()` or similar) and runs on a schedule
3. Check schedule configuration — note the interval (hourly, every N minutes, or configurable)
4. Verify cleanup logic:
   - Queries tower_sessions table
   - Deletes rows where `expiry_date <= now()`
   - Logs number of rows deleted at info level (tracing::info!)
5. Verify task gracefully handles edge case where no sessions exist to clean (does not crash)
6. Write integration test in `tests/session_cleanup.rs`:
   - Insert multiple sessions into tower_sessions with various expiry_date values:
     - Some expired (expiry_date in past)
     - Some valid (expiry_date in future)
   - Manually trigger cleanup task (or wait for scheduled run if testing with time)
   - Query tower_sessions and verify:
     - Only valid sessions remain
     - All expired sessions deleted
   - Capture log output — verify info log contains number of sessions deleted
   - Test edge case: insert NO sessions, trigger cleanup, verify no error and appropriate log message
   - Test multiple runs: cleanup twice in succession, verify second run logs 0 deleted
7. Write unit test for cleanup SQL query:
   - Verify query correctly identifies expiry_date <= now()
   - Use `#[sqlx::test]` for isolated DB testing

**Files:**
- `src/session_cleanup.rs` (read)
- `src/main.rs` (read, verify task is spawned)
- `tests/session_cleanup.rs` (create)

**Test Strategy:**
- Integration test: insert mixed valid/expired sessions, trigger cleanup, verify correct rows deleted
- Integration test: verify info log output contains session count
- Integration test: trigger cleanup on empty table, verify no error
- Unit test: query returns only expired rows (expiry_date <= now())
- Run `cargo test tests::session_cleanup` — should pass
- Optional: Run with `RUST_LOG=fbapp_vibe=info` and verify cleanup logs appear
