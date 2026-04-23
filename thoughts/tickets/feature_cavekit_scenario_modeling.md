---
title: Scenario modeling leaderboard projection
source: context/kits/cavekit-standings.md
source_id: R9
source_status: open
source_title: Scenario Modeling
status: created
phase: Backlog
type: feature
priority: medium
adrs: []
refs: [R1, R2, cavekit-scoring]
created: 2026-04-23
started: ~
completed: ~
tags: [cavekit, standings, scenario, projection]
keywords: [scenario modeling, hypothetical results, projected leaderboard, query params, ephemeral state]
patterns: [in-memory projection, URL state, re-rendered leaderboard]
---

## Summary

Members need to test hypothetical outcomes for unplayed matches and see how the leaderboard would change before those games are decided.

## Acceptance Criteria

- [ ] Unplayed group matches show a hypothetical result picker on the standings page
- [ ] Selecting a hypothetical result re-renders the leaderboard via HTMX
- [ ] Multiple unplayed matches can be hypothesized at once using query params
- [ ] Hypothetical results do not write to the database
- [ ] Clearing the query params restores the actual standings
- [ ] The UI clearly distinguishes actual from projected points
- [ ] Finished matches cannot be hypothesized
- [ ] League membership access control still applies

## Implementation Context

### Relevant files

- `src/modules/standings/handlers.rs` — scenario-aware leaderboard handler
- `src/modules/standings/models.rs` — projection helper model or pure function
- `templates/standings/index.html` and `templates/standings/leaderboard.html` — scenario UI

### ADR constraints

- Scenario state must remain ephemeral and URL-driven
- Projection logic should be applied in memory, not persisted

### Tests

- Unit test the hypothetical projection function
- Integration test query-param driven rendering if practical

### Research Context

#### Keywords to Search

- `scenario modeling` - feature intent
- `hypothetical results` - user input
- `projected leaderboard` - output state
- `query params` - state transport

#### Patterns to Investigate

- URL-driven state for projections
- in-memory recalculation of derived scores
- HTMX re-render flows

#### Key Decisions Made

- Projections are ephemeral and never persisted
- The feature builds on the existing leaderboard rather than a separate view

## Outcome

> Fill this section in after implementation, before moving it to the done archive.

Follow-up tasks: _none_
