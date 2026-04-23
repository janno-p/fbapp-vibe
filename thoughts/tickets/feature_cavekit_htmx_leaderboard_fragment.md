---
title: HTMX leaderboard fragment
source: context/kits/cavekit-standings.md
source_id: R2
source_status: open
source_title: HTMX Leaderboard Fragment
status: created
phase: Backlog
type: feature
priority: medium
adrs: []
refs: [R1]
created: 2026-04-23
started: ~
completed: ~
tags: [cavekit, standings, htmx, leaderboard]
keywords: [HTMX, fragment, leaderboard update, polling, fragment template]
patterns: [fragment rendering, partial refresh, membership guard]
---

## Summary

The leaderboard needs a reusable HTML fragment so the standings can refresh without a full page reload.

## Acceptance Criteria

- [ ] `GET /standings/leaderboard?league_id={id}` returns an HTML fragment without the page wrapper
- [ ] The fragment includes the same columns as the main leaderboard
- [ ] The fragment can be updated by HTMX `hx-get` requests from the client
- [ ] Membership enforcement matches the main leaderboard route
- [ ] The fragment can be reused by polling or user-triggered refreshes

## Implementation Context

### Relevant files

- `src/modules/standings/handlers.rs` — fragment handler
- `templates/standings/leaderboard.html` — fragment template
- `templates/standings/index.html` — host page wiring

### ADR constraints

- Keep fragment rendering separate from full-page rendering
- Preserve the same ranking and access rules as R1

### Tests

- Unit test fragment data rendering if there is reusable formatting logic
- Add an integration test for the fragment route and membership guard

### Research Context

#### Keywords to Search

- `hx-get` - client refresh trigger
- `leaderboard.html` - fragment template
- `fragment` - partial page response
- `polling` - live refresh behavior

#### Patterns to Investigate

- HTMX partials in Askama templates
- shared template data between full page and fragment
- periodic refresh without full navigation

#### Key Decisions Made

- The fragment mirrors R1 instead of introducing a second ranking model
- Refresh behavior is client-driven and not a separate data pipeline

## Outcome

> Fill this section in after implementation, before moving it to the done archive.

Follow-up tasks: _none_
