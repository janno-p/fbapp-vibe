---
title: Per-round leaderboard breakdown
source: context/kits/cavekit-standings.md
source_id: R7
source_status: open
source_title: Per-Round Leaderboard Breakdown
status: created
phase: Backlog
type: feature
priority: medium
adrs: []
refs: [R1, cavekit-scoring]
created: 2026-04-23
started: ~
completed: ~
tags: [cavekit, standings, round, leaderboard]
keywords: [per-round, stage breakdown, group points, knockout points, total]
patterns: [pivoted aggregate table, conditional columns, stage scoring]
---

## Summary

Members need a standings view that breaks total points down by tournament round so performance by stage is visible.

## Acceptance Criteria

- [ ] `GET /leagues/{id}/standings/rounds` renders a per-round breakdown page
- [ ] Only league members can access the page
- [ ] The table shows each member with columns for group stage, knockout rounds, winner bonus, top scorer, and total
- [ ] Each cell shows the points awarded for that round or `—` if not yet scored
- [ ] Rows are sorted by total points descending using the same tie-breaker as R1
- [ ] Only stages with predictions are shown
- [ ] The page is linked from the main leaderboard page

## Implementation Context

### Relevant files

- `src/modules/standings/handlers.rs` — per-round handler
- `src/modules/standings/db.rs` — grouped scoring query
- `src/modules/standings/models.rs` — round breakdown row model
- `templates/standings/rounds.html` — per-round template

### ADR constraints

- Keep the breakdown aligned with the main leaderboard scoring logic
- Avoid duplicating tie-break behavior in multiple places

### Tests

- Integration test for league membership access control
- Unit test any row-pivot or stage-aggregation helper logic

### Research Context

#### Keywords to Search

- `per-round` - page scope
- `winner bonus` - stage-specific column
- `top scorer` - scoring column
- `conditional columns` - hide empty stages

#### Patterns to Investigate

- pivot tables from scored rows
- stage-based aggregation
- dynamic table columns based on data presence

#### Key Decisions Made

- Empty stages are hidden instead of shown as empty columns
- Sorting must remain consistent with the main leaderboard

## Outcome

> Fill this section in after implementation, before moving it to the done archive.

Follow-up tasks: _none_
