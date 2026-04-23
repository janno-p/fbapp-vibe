---
title: Match breakdown page
source: context/kits/cavekit-standings.md
source_id: R3
source_status: open
source_title: Match Breakdown
status: created
phase: Backlog
type: feature
priority: medium
adrs: []
refs: [R1, cavekit-leagues, cavekit-scoring, cavekit-predictions]
created: 2026-04-23
started: ~
completed: ~
tags: [cavekit, standings, match, predictions]
keywords: [match breakdown, predictions, points awarded, consensus, league member]
patterns: [detail page, grouped predictions, access control]
---

## Summary

Members need a per-match breakdown that shows how every league member predicted a fixture and how many points each prediction earned.

## Acceptance Criteria

- [ ] `GET /leagues/{id}/standings/match/{match_id}` renders a match breakdown page
- [ ] Only league members can view the page
- [ ] The page shows league name, match info, and final result when available
- [ ] The table shows each member's prediction and points awarded or pending state
- [ ] Columns include member name, prediction, points awarded, and correct/wrong indicator
- [ ] Rows are sorted by member name or points descending
- [ ] The page shows consensus counts for the prediction split
- [ ] Unknown `match_id` values return blank state or `404`

## Implementation Context

### Relevant files

- `src/modules/standings/handlers.rs` — match breakdown handler
- `src/modules/standings/db.rs` — match-specific prediction query
- `src/modules/standings/models.rs` — match breakdown row model
- `templates/standings/match.html` — detail page template

### ADR constraints

- Keep access control at the route boundary
- Reuse scoring data rather than recomputing match outcomes in the template

### Tests

- Integration test for member-only access
- Integration test for missing `match_id` behavior
- Unit test any consensus or row-sorting helper logic

### Research Context

#### Keywords to Search

- `match breakdown` - page purpose
- `consensus` - prediction split summary
- `points awarded` - scoring output
- `match_id` - route parameter

#### Patterns to Investigate

- detail pages with table summaries
- prediction aggregation by match and user
- null/pending scoring states

#### Key Decisions Made

- Final result is shown only when the match is finished
- The page is league-scoped, not public

## Outcome

> Fill this section in after implementation, before moving it to the done archive.

Follow-up tasks: _none_
