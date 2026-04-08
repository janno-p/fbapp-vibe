---
id: 0041
title: Make knockout prediction form tournament-aware (skip missing rounds)
status: open
phase: MVP
type: bug
adrs: []
refs: []
created: 2026-04-08
started: ~
completed: ~
---

## Goal

The knockout predictions form always shows all six rounds (R32 → Winner) regardless of which rounds the tournament actually has. For a 16-team tournament like UEFA EURO 2024 — which starts at R16 — users see a confusing R32 section where they can submit predictions that will never be scored. The rounds that exist for a given tournament are already stored correctly in the `matches` table (seeded from the API), so the fix is to query those rounds and show only them.

## Acceptance Criteria

- [ ] The knockout predictions form shows only rounds for which matches exist in the tournament's `matches` table
- [ ] A 24/32-team tournament (e.g. EURO 2024 → starts at R16) shows no R32 section
- [ ] A 48-team tournament (WC 2026 → includes R32) shows all rounds including R32
- [ ] The max-achievable-points calculation in the standings leaderboard reflects only rounds that exist in the tournament (R32 does not inflate scores for tournaments that lack it)
- [ ] Existing prediction submissions for rounds that don't exist in a tournament are not affected (no data loss)
- [ ] `cargo test` passes

## Context for Claude 🤖

### Why this happens

`get_knockout_predictions()` (`src/modules/predictions/db.rs`, lines ~137–171) builds `KnockoutRoundState` entries by iterating over `KnockoutRound::all()` — a static slice of all six variants — and merging in whatever the user has already predicted. It never asks "does this tournament actually have R32 matches?".

### The fix: query available rounds from `matches`

Add a new DB helper (or extend the existing query) that fetches the distinct knockout rounds present for a tournament:

```sql
SELECT DISTINCT round AS "round: KnockoutRound"
FROM matches
WHERE tournament_id = $1
  AND round IS NOT NULL
ORDER BY round
```

The `knockout_round` Postgres enum has a stable ordering (`r32 < r16 < qf < sf < final < winner`), so `ORDER BY round` returns them in the correct bracket order without extra sorting in Rust.

Use this result to replace the `KnockoutRound::all()` iterator in `get_knockout_predictions()`. Only return `KnockoutRoundState` entries for rounds that are present — the handler and template need no changes.

> Note: `winner` is stored as a round variant in `matches` for the final match (used for "who wins the whole tournament" prediction). It should be included in the query result so the Winner prediction section is also shown/hidden correctly.

### Where to change

| File | Change |
|---|---|
| `src/modules/predictions/db.rs` | Add `get_tournament_knockout_rounds(pool, tournament_id) -> Result<Vec<KnockoutRound>>` (or inline into `get_knockout_predictions()`). Replace `KnockoutRound::all()` with the DB-derived list. |
| `src/modules/standings/db.rs` | Review the `max_achievable` computation (lines ~135 and ~449). The SQL sums potential points using `SUM(CASE round WHEN 'r32' THEN 2 … END * team_count)` or similar. It must be filtered to only rounds that exist for the tournament — add a JOIN or subquery on `matches` to exclude rounds with no data. |
| `src/modules/standings/models.rs` | Update the test at line ~511 that asserts max achievable points if it hardcodes R32 assumptions — it should parametrize by available rounds. |

### Standings max-achievable concern

The leaderboard query computes `max_achievable` — the maximum points a user could still earn. If R32 isn't in the tournament but the query counts 64 potential R32 points (32 teams × 2 pts), the leaderboard shows inflated ceilings. The fix is to only count rounds that have corresponding matches:

```sql
-- Only include a round's potential points if that round has matches in this tournament
AND kp.round IN (
    SELECT DISTINCT round FROM matches
    WHERE tournament_id = $1 AND round IS NOT NULL
)
```

### What does NOT need to change

- `api_stage_to_round()` in `src/modules/admin/db.rs` — already flexible; handles API variations correctly
- `src/db_types.rs` — `KnockoutRound` enum stays unchanged; all six variants remain valid
- `src/polling/scorer.rs` — the scorer only runs for rounds with actual match outcomes, so it is already tournament-aware
- Migrations — no schema changes needed; `matches.round` is already the source of truth

### ADR constraints

- Use `sqlx::query_as!` or `sqlx::query_scalar!` for the new round-detection query
- Return `anyhow::Result` from DB functions; propagate with `?`

### Tests

- **Unit test** for `get_knockout_predictions()` (or its helper): mock the rounds list and assert that only the supplied rounds appear in the output — no R32 entry when the list starts at R16
- **Update** the standings max-achievable test in `src/modules/standings/models.rs` to cover both a full-bracket (R32 included) and a short-bracket (R16 start) scenario

## Outcome

> Fill this section in after implementation, before moving to `tasks/done/`.

Brief description of what was built, any deviations from the original spec, and follow-up tasks created as a result.

Follow-up tasks: _none_
