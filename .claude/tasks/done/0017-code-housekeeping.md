---
id: 0017
title: Code housekeeping — dead fields, magic constants, session config, admin logging
status: done
type: chore
adrs: []
refs: []
created: 2026-04-07
started: ~
completed: ~
---

## Goal

Several small quality issues found during code review: unused struct fields suppressed with `#[allow(dead_code)]`, a magic number in the top scorer handler, session expiry hardcoded in `main.rs`, an error message that echoes raw user input, and admin actions performed without any audit log. Fix all of these in one pass.

## Acceptance Criteria

- [ ] `session_secret` removed from `Config` (unused) or wired up — if removed, also remove from `.env.example` if present
- [ ] `config` field removed from `AppState` or actually used — if removed, update `AppState::new` / callers
- [ ] Session expiry duration moved to `Config` as `session_duration_hours: u64` with a default of `24`; `main.rs` reads it from config
- [ ] `const TOP_SCORER_PICKS: usize = 3` defined in `src/modules/predictions/handlers.rs` (or `mod.rs`) and used in the handler and error message
- [ ] The `unknown round: {round_slug}` error message in `save_knockout` no longer echoes the raw slug; use a generic message like `"invalid knockout round"`
- [ ] Admin handlers (`activate`, `deactivate`, `lock`, `unlock`, `seed`) log a `tracing::info!` event with the `tournament_id` and acting `user_id`
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test` passes

## Context for Claude 🤖

### Relevant files

- `src/config.rs` — add `session_duration_hours`, remove `session_secret` if unused
- `src/state.rs` — remove `config` field if unused; check `AppState::new` signature
- `src/main.rs` — read `session_duration_hours` from config for `SessionManagerLayer`; remove `mod` declarations that moved to `lib.rs`
- `src/modules/predictions/handlers.rs` — `TOP_SCORER_PICKS` constant, fix error message
- `src/modules/admin/handlers.rs` — add `tracing::info!` to mutating handlers

### ADR constraints

- **ADR-0009**: Error responses to clients should not reflect internal implementation details

### Tests

No new tests needed — these are mechanical cleanups. Existing test suite must still pass.

### Implementation notes

- Check whether `config` in `AppState` is truly unreferenced: `grep -r "state\.config\|\.config\." src/` before removing it. If any handler uses it (e.g. for OAuth client ID), keep it.
- For session duration: `SessionManagerLayer` takes a `Duration` — use `time::Duration::hours(config.session_duration_hours as i64)`.
- Tracing events in admin handlers should use structured fields: `tracing::info!(tournament_id, user_id = user.id, "tournament activated")`.

## Outcome

- `src/config.rs`: removed `session_secret` (was `#[allow(dead_code)]`, never read); added `session_duration_hours` (default 24)
- `src/main.rs`: session expiry now reads `config.session_duration_hours` instead of hardcoded 1
- `.env.example`: removed `SESSION_SECRET`; added optional tuning vars as comments
- `src/modules/predictions/handlers.rs`: added `const TOP_SCORER_PICKS: usize = 3`; used in validation and error message; replaced `"unknown round: {round_slug}"` with `"invalid knockout round"` (ADR-0009)
- `src/modules/admin/handlers.rs`: `activate`, `deactivate`, `lock`, `unlock` handlers now log `tracing::info!(tournament_id, user_id, "...")` after the DB call; changed `_admin` to `admin` to access `admin.0.id`
- `src/modules/admin/mod.rs`: removed `#[allow(dead_code)]` from `AdminUser`
- `tests/admin_routes.rs`: updated `test_config()` to remove `session_secret`, add `session_duration_hours`

Follow-up tasks: _none_
