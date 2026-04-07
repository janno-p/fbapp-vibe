---
id: 0015
title: Validate team and player IDs belong to the tournament
status: open
type: bug
adrs: [0016]
refs: [0007]
created: 2026-04-07
started: ~
completed: ~
---

## Goal

`save_knockout_round_predictions` and `save_top_scorer_predictions` accept team/player IDs from form input and insert them directly without checking they belong to the active tournament. A user could submit IDs from a different tournament (or arbitrary integers) and corrupt prediction data.

## Acceptance Criteria

- [ ] Before inserting knockout predictions, all submitted `team_ids` are verified to exist in `teams` for the given `tournament_id`; any invalid ID returns 400
- [ ] Before inserting top scorer predictions, all submitted `player_ids` are verified to exist in `players` for the given `tournament_id`; any invalid ID returns 400
- [ ] Validation runs inside the same transaction as the insert (after the lock is acquired)
- [ ] `#[sqlx::test]` covers the rejection case for at least one of the two

## Context for Claude 🤖

### Relevant files

- `src/modules/predictions/db.rs` — add validation queries inside `save_knockout_round_predictions` and `save_top_scorer_predictions`, after `assert_predictions_open`
- `src/modules/predictions/handlers.rs` — no changes needed if db layer returns `AppError::BadRequest`

### ADR constraints

- **ADR-0016**: Validation must happen inside the open transaction, after the `FOR UPDATE` lock, so the tournament state cannot change between check and write
- **ADR-0005**: Use `sqlx::query!` for the validation queries

### Tests

`#[sqlx::test]` in `db.rs`:
- Insert a tournament and two teams from different tournaments; attempt to save a knockout prediction with a team_id from the wrong tournament; assert `AppError::BadRequest`

### Implementation notes

Efficient validation query — check that the count of matching rows equals the submitted count:

```sql
SELECT COUNT(*) FROM teams
WHERE tournament_id = $1 AND id = ANY($2)
```

If the returned count differs from `team_ids.len()`, return `AppError::BadRequest("one or more team IDs are not valid for this tournament")`. Same pattern for players.

Use `sqlx::query_scalar!` and pass the id slice as `&team_ids[..]` with `i64` type.

## Outcome

> Fill this section in after implementation, before moving to `tasks/done/`.

Follow-up tasks: _none_
