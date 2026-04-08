---
id: 0029
title: Per-match kickoff countdown timer
status: open
type: feature
adrs: [0003, 0004]
refs: [0021, 0026]
created: 2026-04-08
started: ~
completed: ~
---

## Goal

Match pages and the fixture list show a static formatted kickoff time in UTC, but users have to mentally convert the time themselves. A live countdown ("kicks off in 2h 15m") gives immediate context without requiring time-zone arithmetic and makes the predictions deadline feel tangible.

## Acceptance Criteria

- [ ] Matches that have not yet kicked off show a countdown: "Kicks off in Xh Ym" (or "Xm Ys" when under an hour)
- [ ] When a match has already kicked off the countdown is hidden (element removed or replaced with a "In progress" or "Finished" label depending on whether a result is present)
- [ ] The countdown updates every second in the browser without a page reload
- [ ] The countdown is driven purely by a `data-kickoff-utc` attribute (ISO 8601 / epoch ms) set server-side — no JavaScript date parsing from formatted strings
- [ ] The JavaScript is small, vanilla (no library), and does not block page render
- [ ] Countdown appears on: match breakdown page (`/leagues/{id}/matches/{match_id}`) and fixture list page (task 0026)

## Context for Claude 🤖

### Relevant files

- `templates/standings/match.html` — add `data-kickoff-utc` attribute to the kickoff time element; add a `<span class="js-countdown">` placeholder
- `templates/standings/fixtures.html` (task 0026) — same treatment per match row
- `assets/js/countdown.js` (new file) — vanilla JS that queries `[data-kickoff-utc]` elements, starts `setInterval`, and updates sibling `.js-countdown` spans
- `templates/layout/nav_base.html` or `templates/layout/base.html` — include `countdown.js` (or include it only in the relevant templates via a `{% block scripts %}` extension point)

### ADR constraints

- **ADR-0003**: HTMX is for server-driven partial updates; a client-side countdown is pure UI state that does not require a server round-trip — vanilla JS is the right choice here, not HTMX polling
- **ADR-0004**: Template just needs to emit the UTC epoch as a `data-` attribute; all rendering logic stays in JS

### Tests

- No automated tests — purely client-side UI behaviour; manually verify in browser

### Implementation notes

- Server-side: emit `data-kickoff-utc="{{ match.scheduled_at_epoch_ms() }}"` where `scheduled_at_epoch_ms()` is a new method on `MatchInfo` (or `NearestMatch`) that returns the Unix timestamp in milliseconds as an i64
- Method: `pub fn scheduled_at_epoch_ms(&self) -> i64 { self.scheduled_at.unix_timestamp() * 1000 }`
- JS algorithm: `const diff = target - Date.now()`. If `diff <= 0` hide or replace the countdown. Otherwise compute hours/minutes/seconds and write to the element
- Use `setInterval(tick, 1000)` — acceptable for this precision; no `requestAnimationFrame` needed
- The JS file is served as a static asset from `assets/js/`; check `routes.rs` for the static assets directory (currently `assets/`)
- The layout templates already include Tailwind CSS and HTMX — add the script tag at the bottom of `<body>` (or as a defer script) so it does not block rendering
- Keep the JS under 30 lines; no build step required (vanilla ES6)

## Outcome

Implemented per-match countdown timer using Alpine.js (per ADR-0020) instead of the vanilla JS file originally specified.

- Added `scheduled_at_epoch_ms() -> i64` to `MatchInfo` and `FixtureRow` in `src/modules/standings/models.rs`
- Created `assets/js/countdown.js` — a 13-line Alpine component factory `countdown(kickoffMs)` that computes the label synchronously on construction (no flash) and updates via `setInterval` in `init()`
- Added `{% block head_scripts %}` extension point to `templates/layout/base.html` for page-specific deferred scripts
- Updated `templates/standings/match.html` and `templates/standings/fixtures.html` to use `x-data="countdown(...)"` with `x-text="label"` in the not-played branch

Deviation from spec: Alpine.js used instead of a separate vanilla JS file, consistent with ADR-0020. The `countdown.js` file still exists as a static asset but defines an Alpine component factory rather than direct DOM manipulation.

Follow-up tasks: _none_
