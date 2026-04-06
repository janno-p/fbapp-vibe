---
id: 0005
title: Tournament management (admin)
status: open
type: feature
adrs: [0007, 0009, 0016]
refs: [0004]
created: 2026-04-06
started: ~
completed: ~
---

## Goal

Give admins the ability to register a tournament from the football API, activate it as the current competition, and seed all related data (teams, groups, players, fixtures) into the database. Non-admin users have no access to these screens. This is the prerequisite for all prediction and scoring features.

## Acceptance Criteria

- [ ] Admin-only route group under `/admin`; non-admins receive 403
- [ ] Admin can list available competitions from the API and register one
- [ ] Registering a tournament seeds: teams, groups, group memberships, players, and match fixtures into the DB
- [ ] Admin can activate a tournament (only one active at a time; DB enforces via partial unique index)
- [ ] Admin can set `predictions_locked_at` manually (independent of API — admin decides when to lock)
- [ ] All seed operations are idempotent (re-running does not duplicate rows; uses `INSERT ... ON CONFLICT DO UPDATE`)
- [ ] Admin dashboard shows current tournament status: active/inactive, locked/open, match count, team count

## Context for Claude 🤖

### Relevant files

- `src/modules/admin/mod.rs` — new module, expose `router()`
- `src/modules/admin/handlers.rs` — handler functions
- `src/modules/admin/db.rs` — SQLx queries
- `src/modules/mod.rs` — register admin module
- `src/routes.rs` — mount admin router
- `templates/admin/` — Askama templates
- `migrations/0005_tournament_core.sql` — already written; tables are available

### ADR constraints

- **ADR-0007**: Module lives at `src/modules/admin/`; only `router()` is public
- **ADR-0009**: Return `Result<impl IntoResponse, AppError>`; use `AppError::Unauthorized` for non-admins
- **ADR-0005**: Use `sqlx::query!` macros; all queries compile-time checked
- **ADR-0016**: Seed operations use `external_id` as the conflict key for idempotent upserts

### Auth guard

Check `auth_session.user.is_admin` at the handler level (or via a middleware applied to the admin router). Return `AppError::Unauthorized` if false.

### Tests

- `#[sqlx::test]` integration test for the seed operation: seed a minimal fixture (1 tournament, 2 teams, 1 group, 1 match), re-run the seed with updated data, assert row count stays the same and values are updated (idempotency check).
- `#[sqlx::test]` for the admin auth guard: verify a non-admin user receives 403 on any `/admin` route.
- No unit tests for handlers — logic is thin glue between API client and DB.

### Implementation notes

- Seeding order matters due to FK constraints: tournament → teams → groups → group_memberships → players → matches
- Knockout matches from the API will not have teams assigned initially (TBD slots); store `external_id` and `scheduled_at` only; team IDs are filled in as the tournament progresses
- The admin UI does not need to be polished — functional HTML forms are sufficient
- Do not implement result entry here; that is the polling job (task 0008)
- Player list from the API may be incomplete early in a tournament; re-seeding should update existing rows

## Outcome

> Fill this section in after implementation, before moving to `tasks/done/`.

Follow-up tasks: _none_
