---
title: Member stats page
source: context/kits/cavekit-standings.md
source_id: R6
source_status: open
source_title: Member Stats Page
status: created
phase: Backlog
type: feature
priority: medium
adrs: []
refs: [R1, cavekit-leagues, cavekit-scoring, cavekit-badges]
created: 2026-04-23
started: ~
completed: ~
tags: [cavekit, standings, stats, member]
keywords: [member stats, accuracy, streak, recent form, achievements]
patterns: [profile summary, aggregate statistics, recent activity]
---

## Summary

Members need an individual stats page that summarizes a participant's performance, streaks, and recent predictions.

## Acceptance Criteria

- [ ] `GET /leagues/{id}/members/{user_id}` renders a member stats page
- [ ] Only league members can access the page
- [ ] The page shows member name, avatar, tournament name, total points, and league rank
- [ ] The page shows group stage accuracy and knockout accuracy
- [ ] The page shows top scorer picks
- [ ] The page shows current win streak
- [ ] The page shows the last 5 predictions with results and points
- [ ] Joined date and achievements are shown if available
- [ ] Missing or non-member `user_id` values return `404`

## Implementation Context

### Relevant files

- `src/modules/standings/handlers.rs` — member stats handler
- `src/modules/standings/db.rs` — member aggregate queries
- `src/modules/standings/models.rs` — member stats model
- `templates/standings/member_stats.html` — stats page template

### ADR constraints

- Aggregate stats should be derived from raw prediction data
- Badge/achievement data is optional and should not block the page

### Tests

- Integration test for member-only access
- Integration test for unknown or out-of-league users
- Unit test any streak or accuracy helpers

### Research Context

#### Keywords to Search

- `member stats` - page purpose
- `accuracy` - performance metrics
- `win streak` - derived statistic
- `recent form` - last five predictions

#### Patterns to Investigate

- profile summary pages
- derived metrics from prediction history
- optional achievements rendering

#### Key Decisions Made

- The page is scoped to league membership
- Badges remain optional to avoid coupling the page to unfinished badge work

## Outcome

> Fill this section in after implementation, before moving it to the done archive.

Follow-up tasks: _none_
