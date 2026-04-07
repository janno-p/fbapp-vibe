---
id: 0025
title: League member browser and league metadata page
status: done
type: feature
adrs: []
refs: []
created: 2026-04-07
started: 2026-04-08
completed: 2026-04-08
---

## Goal

League members have no way to see who else is in their league, when the league was created, or how many members it has. The only league-related page is the join flow. A league overview page would give members context and make the social aspect of the app visible.

## Acceptance Criteria

- [ ] `GET /leagues/{id}` renders a league overview page showing: league name, member count, list of member display names, and when the league was created
- [ ] Only members of the league can view the page (non-members get 403)
- [ ] The league overview links to the standings page
- [ ] Admin users can see the invite link/token on the league page (or a separate admin-only section)

## Context for Claude 🤖

### Relevant files

- `src/modules/leagues/mod.rs` — add new route `GET /leagues/{id}`
- `src/modules/leagues/handlers.rs` — add `league_overview` handler
- `src/modules/leagues/db.rs` — add `get_league_with_members(pool, league_id, user_id)` query
- `src/modules/leagues/models.rs` — add `LeagueOverview` struct
- `templates/leagues/overview.html` — new template
- `src/modules/standings/db.rs:is_member` — reuse for membership check (or extract to shared location)

### ADR constraints

- **ADR-0007**: New route in the existing `leagues` module
- **ADR-0009**: Return `AppError::Forbidden` for non-members (not 404 — leaking league existence is acceptable here since users must know the ID to attempt access)

### Tests

- No tests — trivial SELECT query with a membership guard

### Implementation notes

- The `is_member` check in `standings/db.rs` is duplicated. Consider whether to extract it to a shared location (e.g. `src/modules/leagues/db.rs`) and re-export. For now, duplicate is acceptable.
- Member list query: `SELECT u.display_name, lm.joined_at FROM league_members lm JOIN users u ON u.id = lm.user_id WHERE lm.league_id = $1 ORDER BY lm.joined_at ASC`
- Invite token: only show to the league creator or admin users. `leagues.created_by` field should exist — check the schema.

## Outcome

Added `GET /leagues/{id}` league overview page.

- `LeagueMember` and `LeagueOverview` models added to `leagues/models.rs` with `formatted_joined_at()` / `formatted_created_at()` helpers
- `is_member` and `get_league_overview` queries added to `leagues/db.rs`
- `league_overview` handler in `leagues/handlers.rs`: 401 if unauthenticated, 403 if not a member, 404 if league not found; invite token stripped unless viewer is creator or admin
- Route registered: `GET /leagues/{id}`
- Template `templates/leagues/overview.html`: shows name, member count, creation date, member list with join dates, standings link (when nav has an active tournament), invite link section for creator/admin only
- `|length` Askama filter not available in this project — used `.len()` method call instead
