---
id: 0026
title: Tournament fixture list page
status: done
type: feature
adrs: [0007, 0009, 0005]
refs: [0021]
created: 2026-04-08
started: 2026-04-08
completed: 2026-04-08
---

## Goal

Users have no single page to see all matches in the active tournament — who plays who, when, and what the result was. The match breakdown page (`/leagues/{id}/matches/{match_id}`) exists, but there is no index. A fixture list gives every league member an at-a-glance view of the full tournament schedule and live results.

## Acceptance Criteria

- [ ] `GET /leagues/{id}/fixtures` renders a full list of tournament matches grouped by round/stage
- [ ] Each match row shows: home team, away team, kickoff date/time, score (or "TBD" if unplayed)
- [ ] Group stage matches are grouped by match day or group name; knockout matches are grouped by round label
- [ ] Only league members can view the page (non-members get 403; unauthenticated get 401)
- [ ] Each match row links to the existing `/leagues/{id}/matches/{match_id}` breakdown page
- [ ] Page is linked from the league overview (`/leagues/{id}`)

## Context for Claude 🤖

### Relevant files

- `src/modules/standings/handlers.rs` — add `fixture_list` handler; follow the existing `match_breakdown` handler pattern
- `src/modules/standings/db.rs` — add `get_all_matches(pool, tournament_id)` query; tournament_id comes from `nav.active_tournament_id` or a dedicated lookup
- `src/modules/standings/models.rs` — `MatchInfo` struct already exists and has `formatted_kickoff()`; reuse it
- `src/modules/standings/mod.rs` — register new route `GET /leagues/{id}/fixtures`
- `templates/standings/fixtures.html` — new template
- `templates/leagues/overview.html` — add a fixtures link

### ADR constraints

- **ADR-0007**: New route added inside the existing `standings` module (not a new module — fixtures are part of the standings surface)
- **ADR-0009**: Return `AppError::Unauthorized` for unauthenticated, `AppError::Forbidden` for non-members

### Tests

- No tests — the query is a straightforward SELECT with an ORDER BY and the handler is trivial

### Implementation notes

- Matches table: `matches` with columns `id, tournament_id, home_team_id, away_team_id, scheduled_at, home_score, away_score, outcome, round` (verify against schema)
- Knockout round label mapping: `KnockoutRound` enum is in `src/db_types.rs` — use its display or a match arm to produce human-readable labels ("Round of 16", "Quarter-final", etc.)
- Group matches can be ordered by `scheduled_at ASC`; knockout matches by round order then `scheduled_at`
- Consider two sections in the template: "Group Stage" and "Knockout Stage", or group by `round` value
- Re-use the `is_member` check from `leagues/db.rs` (already public after task 0025)
- The `MatchInfo` model already covers what is needed — no new model required
- Active tournament ID: retrieve via `nav.active_tournament_id` (already loaded in `nav::load`) to avoid an extra query

## Outcome

Added `GET /leagues/{id}/fixtures` route in the `standings` module. Implementation:

- **`standings/db.rs`**: `get_all_fixtures(pool, tournament_id)` — queries all matches with team names, scores, group name, and knockout round; sorted group-first (by group name ASC), then knockout by round order, then by kickoff time.
- **`standings/models.rs`**: `FixtureRow`, `FixtureGroup`, and `group_fixtures()` — groups a pre-sorted flat list into labelled sections (e.g. "Group A", "Round of 16").
- **`standings/handlers.rs`**: `fixture_list` handler following the same auth/member pattern as other standings handlers.
- **`standings/mod.rs`**: route registered.
- **`templates/standings/fixtures.html`**: new template showing groups with match rows linking to the match breakdown page; handles no-tournament and no-matches states.
- **`templates/leagues/overview.html`**: "Fixtures" secondary button added alongside "View Standings" (visible only when active tournament exists).

No deviations from spec. Follow-up tasks: _none_
