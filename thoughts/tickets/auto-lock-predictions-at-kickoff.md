---
title: Auto-lock predictions when first tournament match starts
source: .claude/tasks/done/0042-auto-lock-predictions-at-kickoff.md
source_id: 0042
source_status: open
source_title: Auto-lock predictions when first tournament match starts
status: open
phase: MVP
type: feature
adrs: []
refs: [0033]
created: 2026-04-08
started: ~
completed: ~
---

## Summary

Predictions should close automatically when the tournament begins — specifically when the scheduled kickoff time of the first match arrives. At that moment the system sets `predictions_locked_at` on the tournament row, preventing any further changes by users. Administrators retain the ability to manually unlock (and re-lock) via the existing admin dashboard controls for exceptional circumstances. No user action or manual admin intervention is required for the common case.

## Acceptance Criteria

- [ ] When the polling loop runs and detects that the earliest scheduled match of an active tournament has reached its kickoff time, it automatically sets `predictions_locked_at = <first_match_scheduled_at>` if the field is currently NULL
- [ ] The lock timestamp is set to the first match's `scheduled_at` value, not `NOW()` — this is deterministic regardless of when the polling run fires
- [ ] If `predictions_locked_at` is already set (admin locked or previously auto-locked), the auto-lock logic does not overwrite it
- [ ] After auto-lock fires, all prediction submission handlers continue to reject submissions via the existing `assert_predictions_open()` guard — no changes needed there
- [ ] The admin dashboard still shows the "Unlock" button and it still clears `predictions_locked_at` to `NULL` — if the admin unlocks, auto-lock will re-engage on the next polling cycle (unless the admin also sets a future lock time)
- [ ] `cargo test` passes

## Design note — conflict with task 0033

Task 0033 proposes a **per-match revision window**: users can update group stage predictions up until 15 minutes before each individual match. That design is incompatible with the global "lock at first kickoff" rule described here. These two tasks represent different product decisions:

| Approach | Task | Locking granularity |
|---|---|---|
| Global lock at first kickoff | **0042 (this task)** | Tournament-wide, automatic |
| Per-match revision window | 0033 | Per-match, up to 15 min before kickoff |

**Resolution**: Implement 0042 first (simpler, aligns with the stated design constraint). Cancel or refile task 0033 as a future enhancement if the per-match window is ever reconsidered.

## Implementation Context

### What already exists (no changes needed)

- `tournaments.predictions_locked_at TIMESTAMPTZ` — already the source of truth for lock state
- `Tournament::is_predictions_locked()` — already compares `predictions_locked_at <= NOW()`
- `assert_predictions_open()` in `src/modules/predictions/db.rs` — already guards all three submission paths
- Admin `lock_tournament()` and `unlock_tournament()` handlers — already set / clear `predictions_locked_at`
- Admin dashboard UI — already shows lock/unlock buttons and current lock status

### What to add

**1. DB helper — `src/polling/db.rs`** (or `src/modules/admin/db.rs` — whichever owns tournament mutations):

```rust
/// If the tournament's first match has started and predictions_locked_at is NULL,
/// set predictions_locked_at to the first match's scheduled_at.
/// Returns true if the lock was applied, false if already locked or no match has started yet.
pub async fn auto_lock_if_started(pool: &PgPool, tournament_id: i64) -> anyhow::Result<bool> {
    let result = sqlx::query!(
        r#"
        UPDATE tournaments
        SET predictions_locked_at = (
            SELECT MIN(scheduled_at)
            FROM matches
            WHERE tournament_id = $1
        )
        WHERE id = $1
          AND predictions_locked_at IS NULL
          AND EXISTS (
              SELECT 1 FROM matches
              WHERE tournament_id = $1
                AND scheduled_at <= NOW()
          )
        RETURNING id
        "#,
        tournament_id
    )
    .fetch_optional(pool)
    .await?;

    Ok(result.is_some())
}
```

This single `UPDATE … WHERE … RETURNING` pattern is safe under concurrent polling runs: if two polling cycles fire simultaneously, only one will find `predictions_locked_at IS NULL` and win the update.

**2. Call from polling loop — `src/polling/mod.rs`**:

Near the top of `poll()`, after loading the active tournament but before processing match results:

```rust
if db::auto_lock_if_started(&state.pool, tournament.id).await? {
    tracing::info!(tournament_id = tournament.id, "predictions auto-locked: first match started");
}
```

The auto-lock runs on every poll cycle, but the `WHERE predictions_locked_at IS NULL` clause makes it a no-op once the lock is set.

### Edge cases

- **Admin unlocks mid-tournament**: `predictions_locked_at` becomes NULL → the next poll cycle will re-lock automatically. This is intentional: the admin must understand that unlocking during an ongoing tournament will only last until the next poll. If the admin needs a longer unlock window they can disable the polling task or rely on the fact that the next poll may be 60+ seconds away.
- **Tournament not yet seeded** (no matches): the subquery `SELECT MIN(scheduled_at) FROM matches` returns NULL; the `WHERE scheduled_at <= NOW()` EXISTS check fails; no lock is set.
- **All matches scheduled in the future**: the `EXISTS (... WHERE scheduled_at <= NOW())` clause prevents premature locking.

### Tests

- Unit test for `auto_lock_if_started()` using `#[sqlx::test]`:
  - Case 1: tournament with one match in the past → lock is set, function returns true
  - Case 2: tournament with one match in the future → no lock, returns false
  - Case 3: tournament already locked → no overwrite, returns false
  - Case 4: second call after lock is set → idempotent, returns false

### ADR constraints

- Use `sqlx::query!` with `RETURNING` for the conditional update
- Return `anyhow::Result`; errors propagate with `?` in the polling loop

## Outcome

Added `auto_lock_if_started()` to `src/polling/db.rs` — a single `UPDATE … WHERE predictions_locked_at IS NULL … RETURNING id` that atomically sets the lock to `MIN(scheduled_at)` when any match has started. Called it at the top of `poll()` in `src/polling/mod.rs`, before match processing. Added 4 `#[sqlx::test]` integration tests covering: past match locks, future match skips, no-overwrite when already locked, and idempotency. No deviations from spec.

Follow-up tasks: _none_
