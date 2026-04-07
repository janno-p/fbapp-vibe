---
id: 0021
title: Human-friendly match schedule with timezone display
status: open
type: feature
adrs: []
refs: []
created: 2026-04-07
started: ~
completed: ~
---

## Goal

Match kick-off times are stored as `scheduled_at TIMESTAMPTZ` in the DB and fetched from the football API, but they are either not displayed or shown as raw ISO-8601 strings in the predictions page. Users should see dates and times in a readable format, ideally in their local timezone.

## Acceptance Criteria

- [ ] Group stage match cards on `/predictions` show the kick-off date and time
- [ ] Times are displayed in UTC with a clear "UTC" label (server-side rendered, no JS required)
- [ ] If the match has already been played, show "played" or the result rather than a future time
- [ ] The standings match breakdown page shows the kick-off time on the match header

## Context for Claude 🤖

### Relevant files

- `src/modules/predictions/models.rs` — `GroupMatch` struct; `scheduled_at` field is `Option<DateTime<Utc>>` or similar — check exact type
- `src/modules/predictions/db.rs` — query returns `scheduled_at`; confirm it is selected
- `templates/predictions/index.html` — group match card template; add time display
- `templates/standings/match.html` — match header; add time
- `src/modules/standings/db.rs` — `get_match_info` query; check `scheduled_at` is included

### ADR constraints

- **ADR-0005**: Use `sqlx::query!` macros — `scheduled_at` maps to `chrono::DateTime<Utc>` automatically when `chrono` feature is enabled for sqlx

### Tests

- No tests — display-only change

### Implementation notes

- Avoid JavaScript-based timezone conversion (HTMX-only app). Display UTC and let users mentally convert, or add a note like "all times UTC".
- For server-side local time, would need the user's timezone stored in the DB — out of scope. UTC display is the right call for now.
- Askama can format `chrono::DateTime` via `.format("%d %b %H:%M UTC")` — but call this from a helper method on the model struct, not inline in the template, to avoid Askama filter complexity.
- If `scheduled_at` is `None` (not yet announced), show "TBD".
- Future enhancement (separate task): JavaScript `<time datetime="...">` element for browser-local timezone conversion.

## Outcome

_Fill in after completion._
