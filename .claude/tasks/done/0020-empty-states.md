---
id: 0020
title: Graceful empty states (no active tournament)
status: done
type: feature
adrs: []
refs: []
created: 2026-04-07
started: ~
completed: ~
---

## Goal

Several routes return 404 when there is no active tournament instead of showing a friendly message. `/predictions` and `/leagues/{id}/standings` both call `ok_or(AppError::NotFound)` on the active tournament lookup. This is confusing — the route exists, just nothing to show yet. Users before tournament activation see a cryptic 404.

## Acceptance Criteria

- [ ] `/predictions` with no active tournament renders a page saying predictions are not open yet (no 404)
- [ ] `/leagues/{id}/standings` with no active tournament renders a page saying the tournament has not started (no 404)
- [ ] The dashboard (`/dashboard`) shows a clear message when the user has no leagues
- [ ] Empty states link to relevant next actions (join a league, check back later)

## Context for Claude 🤖

### Relevant files

- `src/modules/predictions/handlers.rs:56` — `ok_or(AppError::NotFound)` on active tournament; change to render an empty-state template instead
- `src/modules/standings/handlers.rs` — same pattern; return empty-state template
- `src/modules/auth/handlers.rs` — dashboard handler; check if user has no leagues
- `templates/predictions/index.html` — or add `templates/predictions/no_tournament.html`
- `templates/standings/index.html` — or add `templates/standings/no_tournament.html`

### ADR constraints

- **ADR-0007**: Keep changes within their respective route modules
- **ADR-0009**: Return `Result<impl IntoResponse, AppError>` — returning `Ok(EmptyTemplate)` is fine

### Tests

- No tests needed — these are trivial conditional renders

### Implementation notes

- Simplest approach: add an `Option<Tournament>` field to the existing template structs and branch in the template with `{% if let Some(t) = tournament %}...{% else %}...{% endif %}`.
- Alternative: return early with a separate template struct when tournament is `None`. This is cleaner for large templates.
- Dashboard empty-league state: `db::get_user_leagues` already returns `Vec<League>` — just pass length to template.

## Outcome

- `/predictions` with no active tournament now returns a styled "Predictions aren't open yet" page (`templates/predictions/no_tournament.html`) instead of 404. Handler uses early return pattern with `let Some(...) = ... else { return Ok(NoTournamentTemplate.into_response()) }`.
- `/leagues/{id}/standings` already didn't 404 (returns empty vecs). Added `no_tournament: bool` field to `StandingsTemplate` and `LeaderboardFragment`; leaderboard template now shows "The tournament hasn't started yet" vs "No predictions yet" depending on which case applies.
- Dashboard empty-leagues state was already handled in the existing template.
