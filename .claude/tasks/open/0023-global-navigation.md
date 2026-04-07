---
id: 0023
title: Global navigation bar and breadcrumbs
status: open
type: feature
adrs: []
refs: []
created: 2026-04-07
started: ~
completed: ~
---

## Goal

There is no persistent navigation across the app. Users who are deep in standings or predictions have no visible way to get back to their dashboard or switch leagues without manually editing the URL. The base layout has a minimal header but no nav links.

## Acceptance Criteria

- [ ] Every authenticated page shows a top nav with: app name/logo, link to `/dashboard`, link to `/predictions`, and the current user's display name
- [ ] The active route is visually highlighted in the nav
- [ ] The nav is defined once in `layout/base.html` (not duplicated per template)
- [ ] On mobile the nav collapses or remains usable (Tailwind responsive classes)
- [ ] Unauthenticated pages (home, login) show a minimal header without user links

## Context for Claude 🤖

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

_Fill in after completion._
