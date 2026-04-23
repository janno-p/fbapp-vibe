---
title: Leaderboard tie-breaking secondary sort
source: .claude/tasks/done/0028-leaderboard-tiebreaking.md
source_id: 0028
source_status: done
source_title: Leaderboard tie-breaking secondary sort
status: done
type: feature
adrs: [0005, 0009]
refs: [0009]
created: 2026-04-08
started: 2026-04-08
completed: 2026-04-08
---

## Summary

When two or more league members have the same `total_points`, the leaderboard assigns them the same rank but lists them in arbitrary order (database row order). This is confusing and can feel unfair. A deterministic tie-breaking rule makes rankings unambiguous and gives users a consistent experience.

## Acceptance Criteria

- [ ] Users with equal `total_points` are ranked by `max_achievable DESC` as a tiebreaker (higher ceiling first — they are more likely to overtake)
- [ ] Users with equal `total_points` AND equal `max_achievable` are ranked alphabetically by `user_name ASC` as a final deterministic tiebreaker
- [ ] `build_leaderboard` in `standings/models.rs` reflects the same ordering contract (accepts pre-sorted input, documents the sort key)
- [ ] The leaderboard SQL query in `standings/db.rs` uses `ORDER BY total_points DESC, max_achievable DESC, user_name ASC`
- [ ] Existing unit tests for `build_leaderboard` are updated / extended to cover the tie-breaking case

## Implementation Context

### Relevant files

- `src/modules/standings/db.rs` — update the leaderboard query's `ORDER BY` clause
- `src/modules/standings/models.rs` — `build_leaderboard` and its unit tests; update the doc comment to state the expected sort order of input; add a test case for tied scores

### ADR constraints

- **ADR-0005**: Change is isolated to the SQL ORDER BY; no schema migration needed

### Tests

- Unit test in `standings/models.rs`: add a case where two users share `total_points` and verify the one with higher `max_achievable` appears first
- Unit test: two users share both `total_points` and `max_achievable` — verify alphabetical order

### Implementation notes

- The `build_leaderboard` function assumes input is sorted; it does not re-sort. Update the SQL query and the function's doc comment — no sorting logic needs to change in Rust
- Tie-breaking rule rationale: `max_achievable DESC` is preferred over random because it surfaces the user who can still win, which is the most meaningful sport context. Alphabetical is the neutral final tiebreaker
- The leaderboard SQL is likely in a `query!` or `query_as!` call in `standings/db.rs` — locate the `ORDER BY` clause and extend it
- Assigned ranks: currently `rank: i + 1` (sequential). If desired, players with the exact same `total_points` could share a rank (e.g., two players both get rank 2). This is out of scope for this task — keep sequential ranks for simplicity

## Outcome

The SQL ORDER BY in `standings/db.rs` was already correct (`earned DESC, (earned + possible) DESC, user_name ASC`), so no SQL changes were needed. Updated the `build_leaderboard` doc comment in `standings/models.rs` to document the expected sort contract. Added two new unit tests: `tiebreak_higher_max_achievable_ranks_first` and `tiebreak_equal_points_and_ceiling_sorts_alphabetically`. Both pass.

Follow-up tasks: _none_
