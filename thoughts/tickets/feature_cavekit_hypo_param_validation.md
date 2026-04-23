---
title: Hypothetical param validation
source: context/kits/cavekit-standings.md
source_id: R10
source_status: open
source_title: Scenario Modeling - Hypo Param Validation
status: created
phase: Backlog
type: feature
priority: medium
adrs: []
refs: [R9, R8]
created: 2026-04-23
started: ~
completed: ~
tags: [cavekit, standings, validation, scenario]
keywords: [hypo params, validation, whitelist, truncation, invalid values]
patterns: [server-side validation, whitelist filtering, defensive parsing]
---

## Summary

Hypothetical leaderboard projections need server-side validation so only valid, unplayed group-stage matches can influence the computed projection.

## Acceptance Criteria

- [ ] Non-integer hypothetical match IDs are rejected with `400 Bad Request`
- [ ] Match IDs not in the unplayed group-match whitelist are ignored
- [ ] No more than 20 hypothetical params are processed per request
- [ ] Invalid hypothetical values are ignored unless they are `home`, `draw`, or `away`
- [ ] Knockout match IDs cannot be hypothesized
- [ ] Unit tests cover valid IDs, invalid IDs, knockout IDs, value filtering, and truncation

## Implementation Context

### Relevant files

- `src/modules/standings/handlers.rs` — request parsing and filtering
- `src/modules/standings/models.rs` or helper module — validation helper
- `src/modules/standings/db.rs` — unplayed group-match whitelist source

### ADR constraints

- Validation should be server-side and deterministic
- The whitelist must come from active tournament group matches only

### Tests

- Unit test param parsing and whitelist filtering
- Unit test max-param truncation behavior

### Research Context

#### Keywords to Search

- `hypo params` - query parameter format
- `whitelist` - allowed match IDs source
- `truncation` - max 20 limit
- `server-side validation` - enforcement point

#### Patterns to Investigate

- defensive query-param parsing
- whitelist-based filtering
- error-vs-silent-ignore boundaries

#### Key Decisions Made

- Invalid IDs are rejected or ignored according to type
- Knockout matches are excluded by construction

## Outcome

> Fill this section in after implementation, before moving it to the done archive.

Follow-up tasks: _none_
