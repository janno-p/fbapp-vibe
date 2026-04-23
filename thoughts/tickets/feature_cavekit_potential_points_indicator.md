---
title: Potential points indicator
source: context/kits/cavekit-standings.md
source_id: R11
source_status: open
source_title: Potential Points Indicator
status: created
phase: Backlog
type: feature
priority: high
adrs: []
refs: [R1, R2, cavekit-scoring]
created: 2026-04-23
started: ~
completed: ~
tags: [cavekit, standings, leaderboard, indicator]
keywords: [max_achievable, remaining_possible, visual indicator, Material Symbols, banding]
patterns: [derived ceiling score, visual ranking bands, shared fragment rendering]
---

## Summary

The leaderboard needs a visual ceiling indicator so users can see how much scoring headroom each player still has and how strong their remaining path looks relative to others.

## Acceptance Criteria

- [ ] `max_achievable` is computed for every player and displayed in the existing Max cell
- [ ] `remaining_possible` is displayed as secondary text and does not affect band assignment
- [ ] Band assignment uses absolute `max_achievable` values only
- [ ] The range is split into 7 equal bands based on current render values
- [ ] All players fall into band 4 when the range is zero
- [ ] Each band maps to the correct Material Symbols icon and color
- [ ] Icons are self-hosted, not loaded from a CDN
- [ ] The indicator appears on both the main leaderboard and the HTMX fragment
- [ ] The indicator recomputes correctly on initial render and on fragment refreshes

## Implementation Context

### Relevant files

- `src/modules/standings/models.rs` — ceiling and banding helpers
- `src/modules/standings/handlers.rs` — data passed to the page and fragment
- `templates/standings/index.html` — main leaderboard cell rendering
- `templates/standings/leaderboard.html` — fragment cell rendering
- static asset pipeline for self-hosted Material Symbols font

### ADR constraints

- Keep the indicator inside the existing Max cell; do not add a new column
- Banding must be derived from the current render, not cached

### Tests

- Unit test zero-range behavior
- Unit test band assignment across the 7 ranges
- Unit test icon mapping and display value derivation

### Research Context

#### Keywords to Search

- `max_achievable` - ceiling score source
- `remaining_possible` - secondary display value
- `Material Symbols` - icon set
- `banding` - 7-tier visual grouping

#### Patterns to Investigate

- derived ranking indicators
- self-hosted icon/font assets
- dynamic color-coded band presentation

#### Key Decisions Made

- The visual indicator is derived from `max_achievable`, not from points remaining
- The existing leaderboard layout must not gain another column

## Outcome

> Fill this section in after implementation, before moving it to the done archive.

Follow-up tasks: _none_
