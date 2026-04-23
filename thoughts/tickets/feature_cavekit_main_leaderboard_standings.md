---
title: Main leaderboard standings
source: context/kits/cavekit-standings.md
source_id: R1
source_status: open
source_title: Main Leaderboard
status: created
phase: Backlog
type: feature
priority: high
adrs: []
refs: []
created: 2026-04-23
started: ~
completed: ~
tags: [cavekit, standings, leaderboard]
keywords: [leaderboard, rank, total points, tie-breaker, league membership]
patterns: [ranking, access control, aggregate scoring, empty states]
---

## Summary

League members need a main standings view that ranks everyone by total points for the active tournament and explains where each user sits in the league.

## Acceptance Criteria

- [ ] `GET /leagues/{id}/standings` renders the main leaderboard page
- [ ] Only league members can access the page; non-members receive `401` or `403`
- [ ] The table shows `Rank`, `Name`, `Points`, and optional form/streak fields if available
- [ ] Rows are sorted by total points descending
- [ ] Ties are broken by correct predictions count descending, then user ID ascending
- [ ] Points are summed across all predictions for the active tournament
- [ ] The page shows the league name and member count
- [ ] The page shows a `No tournament active` state when appropriate
- [ ] The page shows a `Predictions locked` indicator and lock time when applicable

## Implementation Context

### Relevant files

- `src/modules/standings/mod.rs` — route registration
- `src/modules/standings/handlers.rs` — main standings handler
- `src/modules/standings/db.rs` — leaderboard query
- `src/modules/standings/models.rs` — leaderboard row and ranking logic
- `templates/standings/index.html` — full page template

### ADR constraints

- Use the existing `standings` module pattern under `src/modules/`
- Keep rank assignment deterministic and computed after query results are loaded
- Preserve league access control before rendering

### Tests

- Unit test the ranking and tie-break ordering logic
- Add an integration test for league membership access control
- Add an integration test for a known points ranking case

### Research Context

#### Keywords to Search

- `standings` - module and route entry point
- `leaderboard` - primary user-facing view
- `correct predictions` - tie-break input
- `active tournament` - data scope for scoring

#### Patterns to Investigate

- aggregation and sorting in Rust
- access control before page rendering
- empty-state rendering for inactive tournaments

#### Key Decisions Made

- Total points are derived from prediction data rather than cached totals
- Tie-breaking is deterministic to avoid unstable rank changes
- League membership is enforced server-side

## Outcome

> Fill this section in after implementation, before moving it to the done archive.

Follow-up tasks: _none_
