---
id: 0032
title: Per-round leaderboard breakdown
status: open
type: feature
adrs: [0007, 0009, 0005]
refs: [0009, 0028]
created: 2026-04-08
started: ~
completed: ~
---

## Goal

The league leaderboard shows cumulative totals but not how points were earned across tournament stages. A round-by-round breakdown lets members see who dominated the group stage, who came alive in the knockouts, and where rankings shifted. This adds narrative to the competition.

## Acceptance Criteria

- [ ] `GET /leagues/{id}/standings/rounds` renders a per-round breakdown table
- [ ] Columns: member name, group stage points, each knockout round points (R16, QF, SF, Final, Winner bonus), top scorer points, total
- [ ] Rows are sorted by total points DESC (same tie-breaking as the main leaderboard — task 0028)
- [ ] Rounds with no predictions yet scored are shown as "—" (NULL / 0 treated as not-yet-scored)
- [ ] Only league members can view the page (401 / 403)
- [ ] Page is linked from the main leaderboard page

## Context for Claude 🤖

### Relevant files

- `src/modules/standings/handlers.rs` — add `round_leaderboard` handler
- `src/modules/standings/db.rs` — add `get_round_breakdown(pool, league_id, tournament_id)` query using conditional aggregation
- `src/modules/standings/models.rs` — add `RoundBreakdownRow` struct with per-stage point fields
- `src/modules/standings/mod.rs` — register route `GET /leagues/{id}/standings/rounds`
- `templates/standings/rounds.html` — new template
- `src/db_types.rs` — `KnockoutRound` enum for round labels

### ADR constraints

- **ADR-0007**: New route inside `standings` module
- **ADR-0005**: Use `query_as!` with `Option<i64>` for nullable aggregated sums

### Tests

- No tests — aggregation query over existing data; handler is trivial

### Implementation notes

- Group stage points: `SUM(gp.points_awarded) FILTER (WHERE gp.points_awarded IS NOT NULL)` joining `group_predictions` with `league_members`
- Knockout points per round: use `CASE` or separate subqueries on `knockout_predictions` filtered by round value
- Top scorer: `SUM(tsp.points_awarded)` from `top_scorer_predictions`
- All joined on `user_id` with `league_members WHERE league_id = $1`
- `KnockoutRound` variants: check `src/db_types.rs` for the enum values and their string representations used in the DB (likely stored as TEXT or an enum type in Postgres)
- Template: a horizontally-scrollable table works well for many columns on mobile
- NULL vs 0: use `Option<i64>` fields in `RoundBreakdownRow`; display `None` as "—" in template using `{% if let Some(pts) = row.group_points %}{{ pts }}{% else %}—{% endif %}`

## Outcome

> Fill this section in after implementation, before moving to `tasks/done/`.

Brief description of what was built, any deviations from the original spec, and follow-up tasks created as a result.

Follow-up tasks: _none_
