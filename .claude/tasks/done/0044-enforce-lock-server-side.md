---
id: 0044
title: Enforce prediction lock server-side in all save handlers
status: done
phase: MVP
type: bug
adrs: []
refs: [0042]
created: 2026-04-09
started: ~
completed: 2026-04-09
---

## Goal

The prediction lock introduced by task 0042 is only enforced in the UI (disabled HTML inputs). All three save handlers (`save_group`, `save_knockout`, `save_top_scorer`) fetch the active tournament but never check `tournament.is_predictions_locked()` before writing to the database. Any user can POST directly to these endpoints after kickoff to modify their predictions, bypassing the lock entirely.

## Acceptance Criteria

- [ ] `save_group` returns `AppError::Forbidden` (or `BadRequest`) when `tournament.is_predictions_locked()` is true
- [ ] `save_knockout` returns the same error when locked
- [ ] `save_top_scorer` returns the same error when locked
- [ ] `cargo test` passes

## Context for Claude 🤖

### Relevant files

- `src/modules/predictions/handlers.rs` — `save_group` (line 101), `save_knockout` (line 127), `save_top_scorer` (line 163)
- `src/modules/predictions/db.rs` — `get_active_tournament` already returns a struct with `predictions_locked_at`
- `src/error.rs` — `AppError` variants

### ADR constraints

- Return `Result<impl IntoResponse, AppError>` from handlers
- Use `AppError::Forbidden` for authenticated users doing something they're not allowed to do; `AppError::BadRequest` is also acceptable here since the lock state is a known business rule

### Tests

No new tests needed — this is a guard condition in a handler, which the project convention says to skip testing (trivial handler wiring). The behaviour is validated by the existing lock mechanism tests.

### Implementation notes

Each handler already calls `db::get_active_tournament` and maps `None` to `AppError::NotFound`. Add the lock check immediately after:

```rust
let tournament = db::get_active_tournament(&state.pool)
    .await?
    .ok_or(AppError::NotFound)?;

if tournament.is_predictions_locked() {
    return Err(AppError::Forbidden);
}
```

All three handlers follow the same pattern, so the fix is three identical additions. The `is_predictions_locked()` method already exists on the tournament model (used by the template).

## Outcome

Added `if tournament.is_predictions_locked() { return Err(AppError::Forbidden); }` to `save_group`, `save_knockout`, and `save_top_scorer` in `handlers.rs`. Also fixed a pre-existing `redundant_static_lifetimes` clippy warning in `src/crests.rs`.

Follow-up tasks: _none_
