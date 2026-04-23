---
title: Hide other users' predictions until tournament is locked
source: .claude/tasks/done/0043-hide-predictions-before-lock.md
source_id: 0043
source_status: done
source_title: Hide other users' predictions until tournament is locked
status: done
phase: MVP
type: feature
adrs: []
refs: [0042]
created: 2026-04-08
started: ~
completed: ~
---

## Summary

League members must not be able to see each other's predictions before the tournament starts (i.e. before `predictions_locked_at` is reached). Seeing others' picks before the lock could influence late submissions and undermines the fairness of the competition. Once the tournament is locked, all prediction data becomes visible as normal — the reveal is part of the game experience.

## Acceptance Criteria

- [ ] `GET /leagues/{id}/standings/compare` returns a "not available yet" page/message when predictions are not locked; full data is shown after lock
- [ ] `GET /leagues/{id}/members/{user_id}` returns a "not available yet" page/message when predictions are not locked; full data is shown after lock
- [ ] The leaderboard (`GET /leagues/{id}/standings`) and fixture list remain accessible before lock — they show no meaningful prediction data before the tournament starts (everyone has 0 points)
- [ ] `GET /leagues/{id}/standings/match/{match_id}` (consensus view) is already lock-gated — **no change needed**
- [ ] Navigation links to the compare and member-stats pages are hidden or visually suppressed before lock so users do not hit a wall when clicking them
- [ ] `cargo test` passes

## Implementation Context

### Current exposure audit

| Route | Handler | Exposes other users' predictions? | Currently gated? |
|---|---|---|---|
| `GET /leagues/{id}/standings` | `standings_page()` | No — points only; all 0 before lock | N/A |
| `GET /leagues/{id}/standings/leaderboard` | `leaderboard_fragment()` | No — points only | N/A |
| `GET /leagues/{id}/standings/match/{match_id}` | `match_breakdown()` | Yes — consensus counts | **Yes, already gated** |
| `GET /leagues/{id}/standings/compare` | `compare_page()` | **Yes — full group predictions of two users** | **No** |
| `GET /leagues/{id}/members/{user_id}` | `member_stats()` | **Yes — accuracy %, streaks, correct/total counts** | **No** |
| `GET /leagues/{id}/predictions/review` | `predictions_review()` | No — current user's own data only | N/A |

### How to gate the two routes

Both handlers are in `src/modules/standings/handlers.rs`. The `Tournament` struct (with `is_predictions_locked()`) is already fetched in these handlers (or can be, since the league lookup already implies the tournament).

**Pattern to follow** — identical to how `match_breakdown()` already handles this:

```rust
// src/modules/standings/handlers.rs — match_breakdown() excerpt (existing, already correct)
if !tournament.is_predictions_locked() {
    return Ok(/* "not available yet" response */);
}
```

Apply the same check at the start of `compare_page()` and `member_stats()`.

### Response for locked-out requests

Return an inline "not yet available" message rather than a redirect or error code. Render a simple template (or reuse an existing partial) that says predictions are visible after the competition begins. HTTP status **200** is fine — this is an expected state, not an error.

If the app has a reusable "empty state" partial or a `no_tournament.html` style template, use it. Otherwise a minimal inline message block is sufficient.

### Navigation suppression

Wherever links to compare or member stats appear in the nav or template, conditionally hide or disable them when `!predictions_locked`:

- The global nav (`templates/layout/`) or the standings page may link to compare — check and gate accordingly
- Member name links in the leaderboard table that navigate to `/members/{user_id}` — suppress or remove `href` when not locked (render as plain text instead of anchor)

The exact locations depend on the template structure; grep for `/compare` and `/members/` in `templates/` to find them.

### Relevant files

- `src/modules/standings/handlers.rs` — `compare_page()` and `member_stats()` — add lock check at the top of each handler
- `src/modules/standings/db.rs` — no changes needed; query is only reached after the lock check passes
- `templates/standings/` — add conditional rendering to hide nav links before lock
- `templates/layout/` — check if any global nav links point to these pages

### Tests

No new unit or integration tests required — the lock-check logic is a one-liner using `is_predictions_locked()` which is already tested. Add a note in the acceptance criteria verification that both routes were manually confirmed to return the "not available" response when `predictions_locked_at IS NULL`.

## Outcome

Added lock gating to `compare_page()` and `member_stats()` in `src/modules/standings/handlers.rs`. Both handlers now call `db::get_active_tournament()` and return a `NotLockedTemplate` (new `templates/standings/not_locked.html`) with HTTP 200 when predictions are not yet locked.

`StandingsTemplate` and `LeaderboardFragment` gained an `is_locked: bool` field — `standings_page()` and `leaderboard_fragment()` now call `get_active_tournament()` instead of `get_active_tournament_id()` to populate it.

Template changes:
- `templates/standings/index.html`: "Compare players →" link wrapped in `{% if is_locked %}` guard
- `templates/standings/leaderboard.html`: member name rendered as plain text when `!is_locked`, anchor when `is_locked`

No deviations from spec. No new tests needed (lock check is a one-liner on already-tested `is_predictions_locked()`).

Follow-up tasks: _none_
