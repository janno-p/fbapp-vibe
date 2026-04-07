---
id: 0016
title: Batch player and group membership inserts during tournament seeding
status: open
type: chore
adrs: []
refs: [0005]
created: 2026-04-07
started: ~
completed: ~
---

## Goal

`seed_tournament_data` in `src/modules/admin/db.rs` issues one SQL query per player and one per group membership inside nested loops. Seeding a 32-team tournament with 26-man squads produces ~850 player inserts plus group membership inserts individually. Batch these with multi-row `UNNEST`-based inserts to reduce round-trips to a fixed number of queries regardless of squad size.

## Acceptance Criteria

- [ ] Player upserts execute as a single query using `UNNEST` (or equivalent bulk insert), not one per player
- [ ] Group membership upserts execute as a single query using `UNNEST`
- [ ] Existing `seed_is_idempotent` test still passes — behaviour is unchanged, only query count differs
- [ ] No new `#[allow(...)]` suppressions introduced

## Context for Claude 🤖

### Relevant files

- `src/modules/admin/db.rs` — `seed_tournament_data`, specifically the `upsert_player` call site in the squad loop and the `upsert_group_membership` call site
- The individual `upsert_player` and `upsert_group_membership` functions can be removed once the callers are updated (check nothing else calls them first)

### ADR constraints

- **ADR-0005**: Use `sqlx::query!`; bulk inserts via UNNEST are supported by SQLx with array parameters

### Tests

The existing `seed_is_idempotent` test covers correctness. No additional tests needed unless the refactor changes observable behaviour.

### Implementation notes

UNNEST pattern for bulk upsert:

```sql
INSERT INTO players (tournament_id, team_id, external_id, name)
SELECT $1, unnest($2::bigint[]), unnest($3::text[]), unnest($4::text[])
ON CONFLICT (tournament_id, external_id) DO UPDATE
    SET name = EXCLUDED.name, team_id = EXCLUDED.team_id
```

Build the parallel arrays before the query by iterating the squads. Same pattern for group memberships. SQLx accepts `&[i64]` and `&[&str]` / `&[String]` as array parameters with the `::bigint[]` / `::text[]` cast.

If any team has an empty squad, the arrays will be empty — `UNNEST` of an empty array is a no-op, which is correct.

## Outcome

> Fill this section in after implementation, before moving to `tasks/done/`.

Follow-up tasks: _none_
