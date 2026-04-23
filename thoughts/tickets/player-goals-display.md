---
title: Display player goal tallies in predictions and standings
source: .claude/tasks/done/0022-player-goals-display.md
source_id: 0022
source_status: done
source_title: Display player goal tallies in predictions and standings
status: done
type: feature
adrs: []
refs: []
created: 2026-04-07
started: 2026-04-07
completed: 2026-04-07
---

## Summary

The `players` table has a `goals_scored` column that is updated by the polling loop, but this data is never surfaced in the UI. The top scorer prediction form just shows player names and teams. Showing live goal tallies helps users track their top scorer picks and makes the feature feel alive during the tournament.

## Acceptance Criteria

- [ ] Top scorer prediction form on `/predictions` shows each player's current `goals_scored` next to their name
- [ ] Players are sorted by `goals_scored DESC, name ASC` in the picker
- [ ] The standings page shows the current top scorer picks with live goal counts
- [ ] A player with 0 goals shows "0" not blank

## Implementation Context

### Relevant files

- `src/modules/predictions/models.rs` — `PlayerInfo` struct; add `goals_scored: i32` field
- `src/modules/predictions/db.rs` — `get_players_with_team` query; already joins players; add `goals_scored` to SELECT and ORDER BY
- `templates/predictions/index.html` — top scorer section; show goal count badge
- `src/modules/standings/db.rs` — may need a query for top scorer display with goal counts
- `templates/standings/index.html` or `leaderboard.html` — show top scorer picks with goals

### ADR constraints

- **ADR-0005**: Extend existing `sqlx::query_as!` — add `goals_scored` to the SELECT

### Tests

- No new tests needed — query is an additive SELECT change
- Existing `PlayerInfo` usages compile-time checked via Askama

### Implementation notes

- `goals_scored` is `INTEGER NOT NULL DEFAULT 0` in the schema — never null, safe to use as `i32`
- Sort order change: currently sorted by name; change to `ORDER BY goals_scored DESC, name ASC`
- The top scorer prediction form must not lock out picking 0-goal players (they may score later)
- Consider a small badge or parenthetical: `"Mbappé (FRA) — 3 goals"` or `"Mbappé · 3"` 

## Outcome

- Added `goals_scored: i32` to `PlayerInfo` struct
- Updated `get_players_with_team()` query to SELECT `p.goals_scored` and sort by `goals_scored DESC, name ASC`
- Added a small indigo badge (`Xg`) next to each player name in the top scorer picker
- `cargo check` passes; sqlx macro validates schema match at compile time
