---
id: 0006
title: Leagues
status: open
type: feature
adrs: [0007, 0009, 0016]
refs: []
created: 2026-04-06
started: ~
completed: ~
---

## Goal

Allow admins to create named leagues and generate invite links, and allow users to join a league via that link. A user may belong to multiple leagues. Leagues provide the competitive grouping used by the leaderboard — users' predictions and scores are global, but the ranking view is per-league.

## Acceptance Criteria

- [ ] Admin can create a league (name); system generates a random `invite_token`
- [ ] Admin can view all leagues and their member counts
- [ ] Each league has a shareable invite URL: `GET /leagues/join/{token}`
- [ ] Visiting the invite URL while logged in adds the user to the league (idempotent — rejoining is a no-op)
- [ ] Visiting the invite URL while logged out redirects to login, then completes the join after auth
- [ ] User's dashboard lists all leagues they belong to
- [ ] Joining a non-existent token returns 404

## Context for Claude 🤖

### Relevant files

- `src/modules/leagues/mod.rs` — new module, expose `router()`
- `src/modules/leagues/handlers.rs`
- `src/modules/leagues/db.rs`
- `src/modules/leagues/models.rs` — `League`, `LeagueMember`
- `src/modules/mod.rs` — register leagues module
- `templates/leagues/` — Askama templates
- `migrations/0006_leagues.sql` — already written

### ADR constraints

- **ADR-0007**: Module at `src/modules/leagues/`
- **ADR-0009**: `AppError::NotFound` for unknown invite token
- **ADR-0016**: `invite_token` is a random opaque string; use `uuid::Uuid::new_v4().to_string()` or similar

### Tests

- `#[sqlx::test]` for join via valid token: creates a league, joins it, asserts `league_members` row exists.
- `#[sqlx::test]` for idempotent join: joining the same league twice does not error and produces one row.
- `#[sqlx::test]` for invalid token: joining with an unknown token returns `AppError::NotFound`.
- No unit tests for handlers — logic is thin.

### Implementation notes

- `invite_token` generation: `uuid` crate (already likely a dependency via sqlx) or `rand` + base62 encoding. Either is fine; UUID v4 is simplest.
- Post-login redirect: store the invite URL in the session before redirecting to login; complete the join in the callback. This requires coordination with the auth module — check how `next` redirect is currently handled, or store in session under a `pending_invite` key.
- League creation is admin-only; join is available to any authenticated user
- Do not implement league deletion in this task

## Outcome

> Fill this section in after implementation, before moving to `tasks/done/`.

Follow-up tasks: _none_
