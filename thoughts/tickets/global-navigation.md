---
title: Global navigation bar and breadcrumbs
source: .claude/tasks/done/0023-global-navigation.md
source_id: 0023
source_status: done
source_title: Global navigation bar and breadcrumbs
status: done
type: feature
adrs: []
refs: []
created: 2026-04-07
started: 2026-04-07
completed: 2026-04-08
---

## Summary

There is no persistent navigation across the app. Users who are deep in standings or predictions have no visible way to get back to their dashboard or switch leagues without manually editing the URL. The base layout has a minimal header but no nav links.

## Acceptance Criteria

- [ ] Every authenticated page shows a top nav with: app name/logo, link to `/dashboard`, link to `/predictions`, and the current user's display name
- [ ] The active route is visually highlighted in the nav
- [ ] The nav is defined once in `layout/base.html` (not duplicated per template)
- [ ] On mobile the nav collapses or remains usable (Tailwind responsive classes)
- [ ] Unauthenticated pages (home, login) show a minimal header without user links

## Implementation Context

### Relevant files

- `templates/layout/base.html` — add nav block here
- `src/modules/auth/handlers.rs` — dashboard, home handlers; check how current user is passed to templates
- Templates that extend base: `predictions/index.html`, `standings/index.html`, `leagues/*.html` — may need user info passed through
- `src/modules/auth/models.rs` or inline — `User` struct fields available

### ADR constraints

- **ADR-0007**: No new module needed — layout change only

### Tests

- No tests — pure template/layout change

### Implementation notes

- The challenge: Askama templates are structs — the base template can only render fields passed from the child template. Either:
  1. Add a `user: Option<User>` field to every page template struct (verbose but explicit)
  2. Use a shared `NavContext { user: Option<User> }` struct embedded in all page templates
  3. Use a custom Askama filter or block — less clear
  Option 2 (shared struct) is cleanest. Define `NavContext` in a shared location and embed it in all page template structs.
- Active route highlighting: pass current path as `active_section: &str` in `NavContext`, match in template with `{% if active_section == "predictions" %}`.
- Keep the nav simple for now: logo + 3 links + username. League switcher is a separate concern.

## Outcome

Added a persistent top navigation bar to all authenticated pages via a new `nav_base.html` intermediate layout that extends `base.html`.

**What was built:**
- `src/nav.rs` — `NavContext` struct with `user_name`, `is_admin`, `current_route`, `standings_league_id`; `nav::load()` returns `anyhow::Result` for compatibility with handler error propagation
- `templates/layout/nav_base.html` — nav bar with Kickoff logo, Dashboard/Predictions/Standings links (Standings shown only when active tournament + league membership), admin links (Tournaments/Leagues when on admin pages), username display, Sign out button
- All authenticated templates updated: `dashboard/index.html`, `predictions/index.html`, `predictions/no_tournament.html`, `standings/index.html`, `standings/match.html`, `standings/compare.html`, `admin/dashboard.html`, `admin/competitions.html`, `admin/leagues.html`
- All corresponding handler structs updated to include `nav: NavContext`, using `tokio::try_join!` for parallel loading where existing queries are present
- All per-page standalone `<header>` elements and `<div class="min-h-screen flex flex-col">` wrappers removed

**Deviations from spec:**
- Used a new intermediate `nav_base.html` layout rather than modifying `base.html` directly — keeps unauthenticated pages (home, login) unaffected without conditional logic
- Admin-specific links (Tournaments, Leagues) are shown only when on admin pages (`current_route == "admin"`), not only when user is_admin — this limits visual noise and keeps the nav uncluttered on non-admin pages
