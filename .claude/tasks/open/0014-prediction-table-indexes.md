---
id: 0014
title: Add indexes to prediction tables
status: open
type: chore
adrs: []
refs: [0007]
created: 2026-04-07
started: ~
completed: ~
---

## Goal

The three prediction tables (`group_stage_predictions`, `knockout_predictions`, `top_scorer_predictions`) and `league_members` have no secondary indexes beyond their primary keys. Every query that loads a user's predictions does a full table scan. Add indexes so reads stay fast as row counts grow.

## Acceptance Criteria

- [ ] New migration adds indexes on all four tables
- [ ] `cargo sqlx migrate run` applies cleanly
- [ ] `cargo test` still passes

## Context for Claude 🤖

### Relevant files

- `migrations/` — add `0012_prediction_indexes.sql` (check the actual next migration number first with `ls migrations/`)
- No Rust changes needed

### ADR constraints

- **ADR-0005**: Schema changes go in versioned migration files

### Tests

No new tests — index existence is a schema-level guarantee verified by migration running without error.

### Implementation notes

Indexes to add:

```sql
-- group_stage_predictions: read by (user_id, match_id) and by (match_id) for score calculation
CREATE INDEX ON group_stage_predictions (user_id, match_id);

-- knockout_predictions: read by (user_id, tournament_id, round)
CREATE INDEX ON knockout_predictions (user_id, tournament_id, round);

-- top_scorer_predictions: read by (user_id, tournament_id)
CREATE INDEX ON top_scorer_predictions (user_id, tournament_id);

-- league_members: read by user_id to list a user's leagues
CREATE INDEX ON league_members (user_id);
```

Check whether `group_stage_predictions` and `knockout_predictions` already have a unique constraint (which implies an index) — if so, skip the duplicate.

## Outcome

> Fill this section in after implementation, before moving to `tasks/done/`.

Follow-up tasks: _none_
