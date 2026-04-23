---
title: Group stage standings calculation with tiebreaker rules
source: .claude/tasks/open/0040-group-stage-standings.md
source_id: 0040
source_status: open
source_title: Group stage standings calculation with tiebreaker rules
status: open
phase: Phase3
type: feature
adrs: []
refs: [0018]
created: 2026-04-08
started: ~
completed: ~
---

## Summary

The API does not provide group stage standings tables. To support scenario modelling (task 0018) and to give users a current view of how each group looks mid-tournament, the application needs to compute group standings from the match results it already stores. Standings must work on partial data (some matches played, some still scheduled), apply the correct football competition tiebreaker rules, and be accessible as a page or component that can later be wired into the scenario modelling feature.

## Acceptance Criteria

- [ ] `src/group_standings.rs` exists as a pure computation module (no DB access, no async) that takes a slice of played match data and returns ranked standings per group
- [ ] Standings correctly compute: MP (matches played), W, D, L, GF (goals for), GA (goals against), GD (goal difference), Pts (3/1/0)
- [ ] Pending matches (outcome `NULL`) are excluded from stats — they contribute to MP only if the implementer decides to show "remaining" games, otherwise skip entirely
- [ ] Teams are sorted by the FIFA World Cup tiebreaker order (see Implementation notes)
- [ ] A DB query function (`src/modules/admin/db.rs` or a new non-route module) fetches all group stage matches with scores for a given tournament
- [ ] A new route renders a group standings page visible to league members (`/leagues/{id}/standings` or `/leagues/{id}/groups`)
- [ ] Unplayed matches are excluded from the stats calculation (a 0-0 unplayed match must not affect GF/GA/GD)
- [ ] `cargo test` passes; pure computation functions have unit tests covering the tiebreaker cases described below

## Implementation Context

### What's already in the database

The `matches` table (migration `0005_tournament_core.sql`) already stores everything needed:

```sql
matches.home_score   INT     -- NULL until match is finished
matches.away_score   INT     -- NULL until match is finished
matches.outcome      match_outcome  -- NULL until match is finished; 'home' | 'draw' | 'away'
matches.group_id     BIGINT  -- non-NULL for group stage matches
```

`group_memberships (group_id, team_id)` links teams to groups. `groups.name` has the display name ("Group A", etc.).

A match should be treated as **played** when `outcome IS NOT NULL` (or equivalently when `home_score IS NOT NULL`). No new columns or migrations are needed.

### Pure computation module — `src/group_standings.rs`

Declare in `src/lib.rs` (or `src/main.rs`, whichever is appropriate). This module must contain:

```rust
pub struct GroupMatchResult {
    pub group_id: i64,
    pub home_team_id: i64,
    pub home_team_name: String,
    pub away_team_id: i64,
    pub away_team_name: String,
    pub home_score: i32,
    pub away_score: i32,
}

pub struct TeamStanding {
    pub team_id: i64,
    pub team_name: String,
    pub mp: i32,   // matches played
    pub wins: i32,
    pub draws: i32,
    pub losses: i32,
    pub gf: i32,   // goals for
    pub ga: i32,   // goals against
    pub gd: i32,   // goal difference (gf - ga)
    pub pts: i32,  // 3 * wins + draws
}

pub struct GroupStandings {
    pub group_name: String,
    pub teams: Vec<TeamStanding>,   // sorted, position 0 = 1st place
}

/// Compute standings for every group from the given finished matches.
/// `all_teams` provides team names for teams with 0 played matches.
pub fn compute_standings(
    matches: &[GroupMatchResult],
    groups: &[(i64, String)],          // (group_id, group_name)
    memberships: &[(i64, i64, String)], // (group_id, team_id, team_name)
) -> Vec<GroupStandings>
```

All inputs are plain data — no DB calls inside this module.

### Tiebreaker order (FIFA World Cup rules)

Apply tiebreakers strictly in this order when two or more teams are equal on points:

1. **Points** (primary sort — descending)
2. **Goal difference** (descending)
3. **Goals for** (descending)
4. **Head-to-head points** — only the match(es) between the tied teams; for 3+ way ties, build a mini-table from H2H results among the tied teams and re-apply steps 1–3 within it
5. **Head-to-head goal difference** (descending)
6. **Head-to-head goals for** (descending)
7. **Alphabetical by team name** (ascending) — deterministic final tiebreaker

> Note: fair play points and drawing of lots are not implemented. The alphabetical fallback is sufficient for display purposes.

### DB query

Add `get_group_stage_match_results(pool, tournament_id) -> Result<Vec<GroupMatchResult>>` that returns only finished matches:

```sql
SELECT
    m.group_id,
    ht.id  AS home_team_id,
    ht.name AS home_team_name,
    at.id  AS away_team_id,
    at.name AS away_team_name,
    m.home_score,
    m.away_score
FROM matches m
JOIN teams ht ON ht.id = m.home_team_id
JOIN teams at ON at.id = m.away_team_id
WHERE m.tournament_id = $1
  AND m.group_id IS NOT NULL
  AND m.outcome IS NOT NULL    -- only finished matches
```

Also add separate queries (or extend existing ones) to fetch groups and group memberships for the tournament so all teams appear in the table even if they haven't played yet.

### Route and template

Add a new route to the `predictions` module (or a new lightweight public module) — wherever it fits best:

- **Path**: `GET /leagues/{league_id}/groups` (or `/leagues/{league_id}/standings`)
- Shows one card per group with the classic standings table: Pos | Team | MP | W | D | L | GF | GA | GD | Pts
- Use the same visual language as the rest of the app (Tailwind, pitch colours, team crests via `find_crest_url`)
- Link it from the league navigation bar

### Tests

Unit-test `compute_standings()` in `#[cfg(test)]` inside `group_standings.rs`. Required cases:

1. **Simple group of 4 — all matches played, clear winner**: verify correct point totals and order
2. **Partial group (3 of 6 matches played)**: teams with no matches show 0/0/0/0, unplayed matches do not affect GF/GA
3. **Points tiebreaker resolved by GD**: two teams equal on points, different goal differences → correct order
4. **Points + GD tie resolved by GF**: verify goals-for breaks the tie
5. **H2H tiebreaker**: two teams equal on all aggregate stats, H2H result determines order
6. **Alphabetical fallback**: perfectly equal teams sorted A→Z

No DB or async tests needed for the computation module. The DB query is straightforward enough to skip integration testing.

### ADR constraints

- Use `sqlx::query_as!` for compile-time query checking
- Handler returns `Result<impl IntoResponse, AppError>`; DB errors map to 500 via `anyhow`
- New route module (if created) must expose only `router() -> Router<AppState>`; declare in `src/modules/mod.rs`

### Implementation notes

- H2H for 3-way ties: build a sub-slice of `GroupMatchResult` that only includes matches between the tied teams, run `compute_standings` on that sub-slice, use the resulting order. Recursion depth is bounded by group size (typically 4).
- UEFA EURO uses a different H2H order (H2H before overall GD) — out of scope. FIFA rules cover the most common case and are sufficient for MVP scenario modelling.
- The `crest_url` field exists on `teams` — consider including it in the DB query and template for visual polish.
- This task is a prerequisite for scenario modelling (task 0018), which will extend this page with hypothetical future results.

## Outcome

> Fill this section in after implementation, before moving it to the done archive.

Brief description of what was built, any deviations from the original spec, and follow-up tasks created as a result.

Follow-up tasks: _none_
