---
title: Member comparison page
source: context/kits/cavekit-standings.md
source_id: R4
source_status: open
source_title: Member Comparison
status: created
phase: Backlog
type: feature
priority: medium
adrs: []
refs: [R1, cavekit-leagues, cavekit-scoring]
created: 2026-04-23
started: ~
completed: ~
tags: [cavekit, standings, compare, predictions]
keywords: [compare, side-by-side, member selector, points, knockout section]
patterns: [comparison table, dual-selection UI, access control]
---

## Summary

Members need a comparison view that places two league participants side-by-side so they can compare predictions, results, and overall performance.

## Acceptance Criteria

- [ ] `GET /leagues/{id}/standings/compare` renders a compare page with a member selector
- [ ] The selector lists all league members
- [ ] Two members can be selected through query params or form submission
- [ ] The comparison table shows all matches with both members' predictions side-by-side
- [ ] Columns include match, actual result, member A prediction, member A points, member B prediction, member B points
- [ ] Correct and incorrect predictions are color-coded
- [ ] Knockout round predictions are shown in a dedicated section
- [ ] Top scorer picks are shown and the actual top scorer is highlighted
- [ ] Invalid member IDs return `404` or a clear error

## Implementation Context

### Relevant files

- `src/modules/standings/handlers.rs` — comparison handler
- `src/modules/standings/db.rs` — paired-member comparison query
- `src/modules/standings/models.rs` — comparison row model
- `templates/standings/compare.html` — compare page template

### ADR constraints

- Keep the comparison view league-scoped
- Reuse existing scoring data and do not duplicate prediction logic in templates

### Tests

- Integration test for two valid league members
- Integration test for invalid member IDs
- Unit test any comparison grouping or presentation helper logic

### Research Context

#### Keywords to Search

- `compare` - route and page intent
- `side-by-side` - layout requirement
- `top scorer` - comparison section
- `knockout` - sectioned prediction data

#### Patterns to Investigate

- two-column comparison tables
- query-param driven selection UIs
- highlighting correct vs incorrect predictions

#### Key Decisions Made

- The selector is limited to league members
- Comparison is a read-only view with no mutation path

## Outcome

> Fill this section in after implementation, before moving it to the done archive.

Follow-up tasks: _none_
