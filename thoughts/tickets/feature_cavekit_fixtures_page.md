---
title: Fixtures page
source: context/kits/cavekit-standings.md
source_id: R5
source_status: open
source_title: Fixtures Page
status: created
phase: Backlog
type: feature
priority: medium
adrs: []
refs: [R3, cavekit-tournament, cavekit-leagues]
created: 2026-04-23
started: ~
completed: ~
tags: [cavekit, standings, fixtures, matches]
keywords: [fixtures, stage grouping, kickoff time, match cards, schedule]
patterns: [grouped list, date headers, linked cards]
---

## Summary

Members need a fixtures page that groups upcoming and completed matches by tournament stage and date.

## Acceptance Criteria

- [ ] `GET /leagues/{id}/fixtures` renders a fixtures page
- [ ] Only league members can access the page
- [ ] Matches are grouped by stage: group stage and knockout rounds
- [ ] Matches are sorted by kickoff time ascending within each stage
- [ ] Each match shows home team, away team, kickoff time, and actual result when finished
- [ ] The next closest match is visually highlighted
- [ ] Match cards link to the match breakdown page
- [ ] Date headers are shown for readability

## Implementation Context

### Relevant files

- `src/modules/standings/handlers.rs` — fixtures handler
- `src/modules/standings/db.rs` — fixtures query
- `templates/standings/fixtures.html` — fixtures page template

### ADR constraints

- Keep stage grouping consistent with tournament data
- Reuse the existing match breakdown route for navigation

### Tests

- Integration test for member-only access
- Unit test any stage-grouping or sort helper logic

### Research Context

#### Keywords to Search

- `fixtures` - page concept
- `kickoff time` - sort order
- `match cards` - presentation unit
- `stage` - group stage and knockout grouping

#### Patterns to Investigate

- grouped schedules with date separators
- linked summary cards
- highlighted next event patterns

#### Key Decisions Made

- Stage grouping should match tournament semantics, not ad hoc labels
- The page is informational and links into match detail views

## Outcome

> Fill this section in after implementation, before moving it to the done archive.

Follow-up tasks: _none_
