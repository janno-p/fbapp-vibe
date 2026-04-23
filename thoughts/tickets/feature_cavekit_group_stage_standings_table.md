---
title: Group stage standings table
source: context/kits/cavekit-standings.md
source_id: R8
source_status: open
source_title: Group Stage Standings Table
status: created
phase: Backlog
type: feature
priority: high
adrs: []
refs: [R9]
created: 2026-04-23
started: ~
completed: ~
tags: [cavekit, standings, groups, table]
keywords: [group standings, MP, W, D, L, GD, H2H, points]
patterns: [pure computation, ranking rules, football tiebreakers]
---

## Summary

The app needs a FIFA-style group standings table that can be computed from finished group matches and rendered for league members.

## Acceptance Criteria

- [ ] `src/group_standings.rs` exists as a pure computation module with no DB or async code
- [ ] The module defines `GroupMatchResult`, `TeamStanding`, and `GroupStandings`
- [ ] `compute_standings()` calculates MP, W, D, L, GF, GA, GD, and points
- [ ] Pending matches are excluded from the standings calculation
- [ ] Teams are sorted by points, goal difference, goals for, head-to-head fields, then alphabetical fallback
- [ ] Head-to-head tiebreaking is implemented when available
- [ ] `GET /leagues/{id}/groups` renders standings for each group
- [ ] Only league members can access the page
- [ ] Pure unit tests cover the major tiebreaker cases

## Implementation Context

### Relevant files

- `src/group_standings.rs` — pure calculation module
- `src/modules/standings/handlers.rs` or adjacent route module — page handler
- `src/modules/standings/db.rs` — match and group data queries
- `templates/standings/groups.html` — group standings page

### ADR constraints

- Computation must remain isolated from DB access
- Query helpers should return the pre-fetched match data needed by the pure function

### Tests

- Unit tests for a simple group, partial group, GD tie, GF tie, H2H tie, and alphabetical fallback
- No DB-backed tests are required for the pure computation module

### Research Context

#### Keywords to Search

- `group standings` - page purpose
- `head-to-head` - tiebreak logic
- `goal difference` - ranking factor
- `compute_standings` - pure function entry point

#### Patterns to Investigate

- pure ranking functions
- football competition tiebreak rules
- isolated domain computation modules

#### Key Decisions Made

- Standings are computed from pre-fetched match results only
- H2H is included as a tiebreaker rather than a separate view concern

## Outcome

> Fill this section in after implementation, before moving it to the done archive.

Follow-up tasks: _none_
